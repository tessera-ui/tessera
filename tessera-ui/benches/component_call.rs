use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicUsize, Ordering},
};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tessera_ui::{
    Callback, State,
    layout::{Stack, layout},
    remember, tessera,
    testing::{self, TestProbe},
};

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

#[tessera]
fn leaf(index: usize, label: u64) {
    black_box((index, label));
    layout(Stack);
}

#[tessera]
fn callback_leaf(index: usize, on_click: Option<Callback>) {
    black_box((index, on_click));
    layout(Stack);
}

#[tessera]
fn leaves(count: usize, callbacks: bool, tick_probe: TestProbe<Option<State<u64>>>) {
    let tick = remember(|| 0_u64);
    tick_probe.set(Some(tick));
    black_box(tick.get());
    layout(Stack).child(move || {
        for index in 0..count {
            if callbacks {
                callback_leaf(index).on_click(move || {
                    black_box(index);
                });
            } else {
                leaf(index, 7);
            }
        }
    });
}

fn benchmark_leaves(criterion: &mut Criterion, count: usize, callbacks: bool) {
    let tick_probe = TestProbe::default();
    let mut session = testing::UiTest::new().content({
        let tick_probe = tick_probe.clone();
        move || leaves(count, callbacks, tick_probe.clone())
    });
    session.frame();
    let tick = tick_probe.get().expect("benchmark invalidation state");
    let mut frame = || {
        session.run(|| tick.with_mut(|value| *value = value.wrapping_add(1)));
        black_box(session.frames(1));
    };
    frame();
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    frame();
    let allocations = ALLOCATIONS.load(Ordering::Relaxed) - before;
    let name = if callbacks { "callback" } else { "value" };
    eprintln!("component_calls/{name}/{count}: {allocations} allocations per frame");
    let mut group = criterion.benchmark_group("component_calls");
    group.bench_function(BenchmarkId::new(name, count), |bencher| {
        bencher.iter(&mut frame)
    });
    group.finish();
}

fn component_calls(criterion: &mut Criterion) {
    benchmark_leaves(criterion, 256, false);
    benchmark_leaves(criterion, 256, true);
}

criterion_group!(benches, component_calls);
criterion_main!(benches);
