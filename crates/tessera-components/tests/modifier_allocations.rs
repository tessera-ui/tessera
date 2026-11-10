use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
};

use tessera_components::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, Modifier,
    layout::NoopDrawModifier,
    modifier::{LayoutModifierNode, ModifierCapabilityExt as _},
};

#[derive(PartialEq)]
struct GenericLayout;

impl LayoutModifierNode for GenericLayout {}

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = COUNTING.try_with(|counting| {
            if counting.get() {
                let _ = ALLOCATIONS.try_with(|total| total.set(total.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
}

fn count_allocations(run: impl FnOnce()) -> usize {
    ALLOCATIONS.with(|total| total.set(0));
    COUNTING.with(|counting| counting.set(true));
    run();
    COUNTING.with(|counting| counting.set(false));
    ALLOCATIONS.with(Cell::get)
}

#[test]
fn two_action_construction_allocates_nodes_but_clones_do_not() {
    let generic = count_allocations(|| {
        for _ in 0..3_000 {
            black_box(
                Modifier::new()
                    .push_layout(GenericLayout)
                    .push_draw(NoopDrawModifier),
            );
        }
    });
    assert!(
        generic > 0,
        "the allocator probe must observe generic action allocations"
    );
    let generic_shared = Modifier::new()
        .push_layout(GenericLayout)
        .push_draw(NoopDrawModifier);
    let generic_clones = count_allocations(|| {
        for _ in 0..3_000 {
            black_box(generic_shared.clone());
        }
    });
    assert_eq!(generic_clones, 0, "3,000 clones of a generic chain");

    let make = || {
        Modifier::new()
            .size(Dp(24.0), Dp(13.0))
            .background(Color::RED)
    };
    black_box(make());
    let allocations = count_allocations(|| {
        for _ in 0..3_000 {
            black_box(make());
        }
    });
    assert_eq!(
        allocations, generic,
        "same storage cost for two generic actions"
    );

    let shared = make();
    let allocations = count_allocations(|| {
        for _ in 0..3_000 {
            black_box(shared.clone());
        }
    });
    assert_eq!(allocations, 0, "3,000 clones of one chain");
    eprintln!(
        "3,000 generic two-action chains: {generic} allocations; size/background: {allocations}; generic clones: {generic_clones}; size/background clones: 0"
    );
}
