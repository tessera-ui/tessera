use std::{
    collections::{HashMap, HashSet, VecDeque},
    panic::{self, AssertUnwindSafe},
    sync::{Condvar, Mutex},
    thread,
};

use ra_ap_base_db::Crate;
use ra_ap_hir::{TraitImpls, crate_def_map};
use ra_ap_ide::RootDatabase;

struct Schedule {
    ready: VecDeque<Crate>,
    waiting: HashMap<Crate, usize>,
    running: usize,
}

pub(super) fn prime(db: &RootDatabase, crates: &[Crate], workers: usize) {
    let scope = crates
        .iter()
        .flat_map(|krate| krate.transitive_deps(db))
        .collect::<HashSet<_>>();
    let mut dependents = HashMap::<_, Vec<_>>::new();
    let mut schedule = Schedule {
        ready: VecDeque::new(),
        waiting: HashMap::new(),
        running: 0,
    };
    for &krate in &scope {
        let dependencies = krate
            .data(db)
            .dependencies
            .iter()
            .map(|dependency| dependency.crate_id)
            .filter(|dependency| scope.contains(dependency))
            .collect::<HashSet<_>>();
        for &dependency in &dependencies {
            dependents.entry(dependency).or_default().push(krate);
        }
        if dependencies.is_empty() {
            schedule.ready.push_back(krate);
        } else {
            schedule.waiting.insert(krate, dependencies.len());
        }
    }
    let schedule = Mutex::new(schedule);
    let changed = Condvar::new();
    thread::scope(|scope| {
        for _ in 0..workers.max(1) {
            let db = db.clone();
            let (schedule, changed, dependents) = (&schedule, &changed, &dependents);
            scope.spawn(move || {
                loop {
                    let mut state = schedule.lock().expect("warm-up schedule poisoned");
                    let krate = loop {
                        if let Some(krate) = state.ready.pop_front() {
                            state.running += 1;
                            break Some(krate);
                        }
                        if state.running == 0 {
                            break None;
                        }
                        state = changed.wait(state).expect("warm-up schedule poisoned");
                    };
                    drop(state);
                    let Some(krate) = krate else {
                        changed.notify_all();
                        return;
                    };
                    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                        ra_ap_hir::attach_db(&db, || _ = crate_def_map(&db, krate))
                    }));
                    super::watchdog::progress();
                    let mut state = schedule.lock().expect("warm-up schedule poisoned");
                    state.running -= 1;
                    for dependent in dependents.get(&krate).into_iter().flatten() {
                        if let Some(count) = state.waiting.get_mut(dependent) {
                            *count -= 1;
                            if *count == 0 {
                                state.waiting.remove(dependent);
                                state.ready.push_back(*dependent);
                            }
                        }
                    }
                    drop(state);
                    changed.notify_all();
                    // Method resolution reads the impls of every dependency.
                    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                        ra_ap_hir::attach_db(&db, || _ = TraitImpls::for_crate(&db, krate))
                    }));
                }
            });
        }
    });
}
