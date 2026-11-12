use std::{
    hint::black_box,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tessera_ui::{
    State,
    layout::Stack,
    layout::layout,
    remember, remember_with_key, tessera,
    testing::TestProbe,
    testing::{self},
};

#[tessera]
fn remembered_slot_workload(
    initial_keys: Arc<[usize]>,
    keys_probe: TestProbe<Option<State<Arc<[usize]>>>>,
    tick_probe: TestProbe<Option<State<u64>>>,
    builds: Arc<AtomicUsize>,
) {
    let keys = remember(|| initial_keys);
    let tick = remember(|| 0_u64);
    keys_probe.set(Some(keys));
    tick_probe.set(Some(tick));
    black_box(tick.get());
    let keys = keys.get();
    builds.fetch_add(1, Ordering::Relaxed);
    for &key in keys.iter() {
        black_box(remember_with_key(key, || key));
    }
    layout(Stack);
}

fn benchmark_slot_workload(criterion: &mut Criterion, count: usize, structural_edit: bool) {
    let forward: Arc<[usize]> = (0..count).collect::<Vec<_>>().into();
    let mut local_edit = (0..count).collect::<Vec<_>>();
    if count > 1 {
        local_edit.swap(count / 2, count / 2 - 1);
    }
    let local_edit: Arc<[usize]> = local_edit.into();
    let keys_probe = TestProbe::default();
    let tick_probe = TestProbe::default();
    let builds = Arc::new(AtomicUsize::new(0));
    let initial_keys = forward.clone();
    let mut session = testing::UiTest::new().content(|| {
        remembered_slot_workload(
            initial_keys.clone(),
            keys_probe.clone(),
            tick_probe.clone(),
            builds.clone(),
        )
    });
    session.frame();
    let keys = keys_probe.get().expect("benchmark key state");
    let tick = tick_probe.get().expect("benchmark invalidation state");
    let mut was_edited = false;
    let name = if structural_edit {
        "local_edit"
    } else {
        "stable_recompose"
    };
    let mut group = criterion.benchmark_group("remembered_slots");
    group.bench_function(BenchmarkId::new(name, count), |bencher| {
        bencher.iter(|| {
            session.run(|| {
                tick.with_mut(|value| *value = value.wrapping_add(1));
                if structural_edit {
                    was_edited = !was_edited;
                    keys.set(if was_edited {
                        local_edit.clone()
                    } else {
                        forward.clone()
                    });
                }
            });
            let previous_builds = builds.load(Ordering::Relaxed);
            black_box(session.frames(1));
            let current_builds = builds.load(Ordering::Relaxed);
            assert!(
                current_builds > previous_builds,
                "the benchmark must rebuild"
            );
        });
    });
    group.finish();
}

fn slot_table(criterion: &mut Criterion) {
    benchmark_slot_workload(criterion, 128, false);
    benchmark_slot_workload(criterion, 128, true);
    benchmark_slot_workload(criterion, 1024, false);
    benchmark_slot_workload(criterion, 1024, true);
}

criterion_group!(benches, slot_table);
criterion_main!(benches);
