use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use parking_lot::RwLock;

use crate::{
    ComponentNode, ComponentNodeMetaDatas, ComponentNodeTree, ComponentTree,
    ComputeResourceManager, ComputedData, Constraint, DimensionValue, LayoutInput, LayoutOutput,
    LayoutSpec, MeasurementError, NodeId, Px, PxPosition,
    component_tree::{LayoutContext, measure_node},
    layout::LayoutSpecDyn,
    runtime::LayoutCache,
};

#[derive(Clone)]
struct StackLayout {
    id: u32,
    calls: Arc<AtomicUsize>,
}

impl StackLayout {
    fn new(id: u32, calls: Arc<AtomicUsize>) -> Self {
        Self { id, calls }
    }
}

impl PartialEq for StackLayout {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl LayoutSpec for StackLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        self.calls.fetch_add(1, Ordering::SeqCst);

        if input.children_ids().is_empty() {
            return Ok(ComputedData::min_from_constraint(
                input.parent_constraint().as_ref(),
            ));
        }

        let nodes_to_measure = input
            .children_ids()
            .iter()
            .map(|&child_id| (child_id, *input.parent_constraint().as_ref()))
            .collect();
        let sizes = input.measure_children(nodes_to_measure)?;

        let mut final_width = Px(0);
        let mut final_height = Px(0);
        for (child_id, size) in sizes {
            output.place_child(child_id, PxPosition::ZERO);
            final_width = final_width.max(size.width);
            final_height = final_height.max(size.height);
        }

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }
}

#[derive(Clone)]
struct FixedLayout {
    id: u32,
    size: ComputedData,
    calls: Arc<AtomicUsize>,
}

impl FixedLayout {
    fn new(id: u32, size: ComputedData, calls: Arc<AtomicUsize>) -> Self {
        Self { id, size, calls }
    }
}

impl PartialEq for FixedLayout {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.size == other.size
    }
}

impl LayoutSpec for FixedLayout {
    fn measure(
        &self,
        _input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.size)
    }
}

#[derive(Clone)]
struct OffsetLayout {
    id: u32,
    offset: PxPosition,
    calls: Arc<AtomicUsize>,
}

impl OffsetLayout {
    fn new(id: u32, offset: PxPosition, calls: Arc<AtomicUsize>) -> Self {
        Self { id, offset, calls }
    }
}

impl PartialEq for OffsetLayout {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.offset == other.offset
    }
}

impl LayoutSpec for OffsetLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        self.calls.fetch_add(1, Ordering::SeqCst);

        if input.children_ids().is_empty() {
            return Ok(ComputedData::min_from_constraint(
                input.parent_constraint().as_ref(),
            ));
        }

        let child_id = input.children_ids()[0];
        let size = input.measure_child_in_parent_constraint(child_id)?;
        output.place_child(child_id, self.offset);
        Ok(size)
    }
}

fn create_device() -> Option<wgpu::Device> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter_result =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: true,
        }));
    let adapter = match adapter_result {
        Ok(adapter) => adapter,
        Err(_) => return None,
    };
    let device_result = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        label: None,
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::default(),
    }));
    match device_result {
        Ok((device, _queue)) => Some(device),
        Err(_) => None,
    }
}

fn fixed_constraint(width: i32, height: i32) -> Constraint {
    Constraint::new(
        DimensionValue::Fixed(Px::new(width)),
        DimensionValue::Fixed(Px::new(height)),
    )
}

fn build_two_child_tree(
    root_key: u64,
    child1_key: u64,
    child2_key: u64,
    root_spec: Box<dyn LayoutSpecDyn>,
    child1_spec: Box<dyn LayoutSpecDyn>,
    child2_spec: Box<dyn LayoutSpecDyn>,
) -> (ComponentTree, NodeId, NodeId, NodeId) {
    let mut tree = ComponentTree::new();
    let root_id = tree.add_node(ComponentNode {
        fn_name: "root".to_string(),
        logic_id: root_key,
        instance_key: root_key,
        input_handler_fn: None,
        layout_spec: root_spec,
    });
    let child1_id = tree.add_node(ComponentNode {
        fn_name: "child1".to_string(),
        logic_id: child1_key,
        instance_key: child1_key,
        input_handler_fn: None,
        layout_spec: child1_spec,
    });
    tree.pop_node();
    let child2_id = tree.add_node(ComponentNode {
        fn_name: "child2".to_string(),
        logic_id: child2_key,
        instance_key: child2_key,
        input_handler_fn: None,
        layout_spec: child2_spec,
    });
    tree.pop_node();
    tree.pop_node();
    (tree, root_id, child1_id, child2_id)
}

fn build_single_child_tree(
    root_key: u64,
    child_key: u64,
    root_spec: Box<dyn LayoutSpecDyn>,
    child_spec: Box<dyn LayoutSpecDyn>,
) -> (ComponentTree, NodeId, NodeId) {
    let mut tree = ComponentTree::new();
    let root_id = tree.add_node(ComponentNode {
        fn_name: "root".to_string(),
        logic_id: root_key,
        instance_key: root_key,
        input_handler_fn: None,
        layout_spec: root_spec,
    });
    let child_id = tree.add_node(ComponentNode {
        fn_name: "child".to_string(),
        logic_id: child_key,
        instance_key: child_key,
        input_handler_fn: None,
        layout_spec: child_spec,
    });
    tree.pop_node();
    tree.pop_node();
    (tree, root_id, child_id)
}

fn measure_root(
    root_id: NodeId,
    constraint: &Constraint,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    resource_manager: Arc<RwLock<ComputeResourceManager>>,
    gpu: &wgpu::Device,
    layout_ctx: Option<&LayoutContext<'_>>,
) -> ComputedData {
    match measure_node(
        root_id,
        constraint,
        tree,
        metadatas,
        resource_manager,
        gpu,
        layout_ctx,
    ) {
        Ok(size) => size,
        Err(err) => panic!("measure_node failed: {err:?}"),
    }
}

#[test]
fn layout_cache_reuses_pure_layout_subtree() {
    let Some(gpu) = create_device() else {
        return;
    };

    let root_key = 1;
    let child1_key = 2;
    let child2_key = 3;

    let root_calls = Arc::new(AtomicUsize::new(0));
    let child1_calls = Arc::new(AtomicUsize::new(0));
    let child2_calls = Arc::new(AtomicUsize::new(0));

    let root_spec = StackLayout::new(1, root_calls.clone());
    let child1_size = ComputedData {
        width: Px::new(10),
        height: Px::new(20),
    };
    let child2_size = ComputedData {
        width: Px::new(30),
        height: Px::new(40),
    };
    let child1_spec = FixedLayout::new(1, child1_size, child1_calls.clone());
    let child2_spec = FixedLayout::new(1, child2_size, child2_calls.clone());

    let (tree, root_id, child1_id, child2_id) = build_two_child_tree(
        root_key,
        child1_key,
        child2_key,
        Box::new(root_spec),
        Box::new(child1_spec),
        Box::new(child2_spec),
    );

    let layout_cache = LayoutCache::default();
    let constraint = fixed_constraint(100, 100);
    let resource_manager = Arc::new(RwLock::new(ComputeResourceManager::new()));

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 1,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child1_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child2_calls.load(Ordering::SeqCst), 1);

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 2,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child1_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child2_calls.load(Ordering::SeqCst), 1);

    let child1_meta = match tree.metadatas().get(&child1_id) {
        Some(meta) => meta,
        None => panic!("child1 metadata missing"),
    };
    assert_eq!(child1_meta.rel_position, Some(PxPosition::ZERO));
    assert_eq!(child1_meta.computed_data, Some(child1_size));
    drop(child1_meta);

    let child2_meta = match tree.metadatas().get(&child2_id) {
        Some(meta) => meta,
        None => panic!("child2 metadata missing"),
    };
    assert_eq!(child2_meta.rel_position, Some(PxPosition::ZERO));
    assert_eq!(child2_meta.computed_data, Some(child2_size));
}

#[test]
fn layout_cache_invalidates_on_constraint_change() {
    let Some(gpu) = create_device() else {
        return;
    };

    let root_key = 10;
    let child1_key = 11;
    let child2_key = 12;

    let root_calls = Arc::new(AtomicUsize::new(0));
    let child1_calls = Arc::new(AtomicUsize::new(0));
    let child2_calls = Arc::new(AtomicUsize::new(0));

    let root_spec = StackLayout::new(1, root_calls.clone());
    let child1_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(8),
            height: Px::new(12),
        },
        child1_calls.clone(),
    );
    let child2_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(16),
            height: Px::new(18),
        },
        child2_calls.clone(),
    );

    let (tree, root_id, _child1_id, _child2_id) = build_two_child_tree(
        root_key,
        child1_key,
        child2_key,
        Box::new(root_spec),
        Box::new(child1_spec),
        Box::new(child2_spec),
    );

    let layout_cache = LayoutCache::default();
    let resource_manager = Arc::new(RwLock::new(ComputeResourceManager::new()));
    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 1,
    };

    let constraint = fixed_constraint(200, 200);
    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 1);

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 2,
    };

    let constraint = fixed_constraint(320, 240);
    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 2);
    assert_eq!(child1_calls.load(Ordering::SeqCst), 2);
    assert_eq!(child2_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn layout_cache_invalidates_on_layout_spec_change() {
    let Some(gpu) = create_device() else {
        return;
    };

    let root_key = 21;
    let child1_key = 22;
    let child2_key = 23;

    let root_calls = Arc::new(AtomicUsize::new(0));
    let root_calls_next = Arc::new(AtomicUsize::new(0));
    let child1_calls = Arc::new(AtomicUsize::new(0));
    let child2_calls = Arc::new(AtomicUsize::new(0));

    let root_spec = StackLayout::new(1, root_calls.clone());
    let child1_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(14),
            height: Px::new(24),
        },
        child1_calls.clone(),
    );
    let child2_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(18),
            height: Px::new(28),
        },
        child2_calls.clone(),
    );

    let (mut tree, root_id, _child1_id, _child2_id) = build_two_child_tree(
        root_key,
        child1_key,
        child2_key,
        Box::new(root_spec),
        Box::new(child1_spec),
        Box::new(child2_spec),
    );

    let layout_cache = LayoutCache::default();
    let resource_manager = Arc::new(RwLock::new(ComputeResourceManager::new()));
    let constraint = fixed_constraint(120, 120);

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 1,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 1);

    let Some(root_node) = tree.get_mut(root_id) else {
        panic!("root node missing");
    };
    root_node.layout_spec = Box::new(StackLayout::new(2, root_calls_next.clone()));

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 2,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 1);
    assert_eq!(root_calls_next.load(Ordering::SeqCst), 1);
}

#[test]
fn layout_cache_invalidates_on_child_size_change() {
    let Some(gpu) = create_device() else {
        return;
    };

    let root_key = 31;
    let child_key = 32;

    let root_calls = Arc::new(AtomicUsize::new(0));
    let child_calls = Arc::new(AtomicUsize::new(0));

    let root_spec = StackLayout::new(1, root_calls.clone());
    let child_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(10),
            height: Px::new(12),
        },
        child_calls.clone(),
    );

    let (mut tree, root_id, child_id) = build_single_child_tree(
        root_key,
        child_key,
        Box::new(root_spec),
        Box::new(child_spec),
    );

    let layout_cache = LayoutCache::default();
    let resource_manager = Arc::new(RwLock::new(ComputeResourceManager::new()));
    let constraint = fixed_constraint(160, 160);

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 1,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child_calls.load(Ordering::SeqCst), 1);

    let Some(child_node) = tree.get_mut(child_id) else {
        panic!("child node missing");
    };
    child_node.layout_spec = Box::new(FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(14),
            height: Px::new(18),
        },
        child_calls.clone(),
    ));

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 2,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 2);
    assert_eq!(child_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn layout_cache_skips_when_children_not_measured() {
    let Some(gpu) = create_device() else {
        return;
    };

    let root_key = 35;
    let child1_key = 36;
    let child2_key = 37;

    let root_calls = Arc::new(AtomicUsize::new(0));
    let child1_calls = Arc::new(AtomicUsize::new(0));
    let child2_calls = Arc::new(AtomicUsize::new(0));

    let root_spec = OffsetLayout::new(1, PxPosition::ZERO, root_calls.clone());
    let child1_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(18),
            height: Px::new(22),
        },
        child1_calls.clone(),
    );
    let child2_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(26),
            height: Px::new(30),
        },
        child2_calls.clone(),
    );

    let (tree, root_id, _child1_id, _child2_id) = build_two_child_tree(
        root_key,
        child1_key,
        child2_key,
        Box::new(root_spec),
        Box::new(child1_spec),
        Box::new(child2_spec),
    );

    let layout_cache = LayoutCache::default();
    let resource_manager = Arc::new(RwLock::new(ComputeResourceManager::new()));
    let constraint = fixed_constraint(200, 200);

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 1,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 2,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 2);
    assert_eq!(child1_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child2_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn layout_cache_invalidates_on_child_list_change() {
    let Some(gpu) = create_device() else {
        return;
    };

    let root_key = 41;
    let child1_key = 42;
    let child2_key = 43;

    let root_calls = Arc::new(AtomicUsize::new(0));
    let child1_calls = Arc::new(AtomicUsize::new(0));
    let child2_calls = Arc::new(AtomicUsize::new(0));

    let root_spec = StackLayout::new(1, root_calls.clone());
    let child1_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(12),
            height: Px::new(16),
        },
        child1_calls.clone(),
    );

    let (tree_frame1, root_id_frame1, _child_id_frame1) = build_single_child_tree(
        root_key,
        child1_key,
        Box::new(root_spec),
        Box::new(child1_spec),
    );

    let layout_cache = LayoutCache::default();
    let resource_manager = Arc::new(RwLock::new(ComputeResourceManager::new()));
    let constraint = fixed_constraint(150, 150);

    let layout_ctx_frame1 = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 1,
    };

    let _ = measure_root(
        root_id_frame1,
        &constraint,
        tree_frame1.tree(),
        tree_frame1.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx_frame1),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child1_calls.load(Ordering::SeqCst), 1);

    let root_spec = StackLayout::new(1, root_calls.clone());
    let child1_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(12),
            height: Px::new(16),
        },
        child1_calls.clone(),
    );
    let child2_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(20),
            height: Px::new(24),
        },
        child2_calls.clone(),
    );

    let (tree_frame2, root_id_frame2, _child1_id_frame2, _child2_id_frame2) = build_two_child_tree(
        root_key,
        child1_key,
        child2_key,
        Box::new(root_spec),
        Box::new(child1_spec),
        Box::new(child2_spec),
    );

    let layout_ctx_frame2 = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 2,
    };

    let _ = measure_root(
        root_id_frame2,
        &constraint,
        tree_frame2.tree(),
        tree_frame2.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx_frame2),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 2);
    assert_eq!(child1_calls.load(Ordering::SeqCst), 1);
    assert_eq!(child2_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn layout_cache_updates_child_placement_after_layout_change() {
    let Some(gpu) = create_device() else {
        return;
    };

    let root_key = 51;
    let child_key = 52;

    let root_calls = Arc::new(AtomicUsize::new(0));
    let child_calls = Arc::new(AtomicUsize::new(0));

    let initial_offset = PxPosition::new(Px::new(0), Px::new(0));
    let root_spec = OffsetLayout::new(1, initial_offset, root_calls.clone());
    let child_spec = FixedLayout::new(
        1,
        ComputedData {
            width: Px::new(24),
            height: Px::new(28),
        },
        child_calls.clone(),
    );

    let (mut tree, root_id, child_id) = build_single_child_tree(
        root_key,
        child_key,
        Box::new(root_spec),
        Box::new(child_spec),
    );

    let layout_cache = LayoutCache::default();
    let resource_manager = Arc::new(RwLock::new(ComputeResourceManager::new()));
    let constraint = fixed_constraint(120, 120);

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 1,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    let child_meta = match tree.metadatas().get(&child_id) {
        Some(meta) => meta,
        None => panic!("child metadata missing"),
    };
    assert_eq!(child_meta.rel_position, Some(initial_offset));
    drop(child_meta);

    let updated_offset = PxPosition::new(Px::new(12), Px::new(34));
    let Some(root_node) = tree.get_mut(root_id) else {
        panic!("root node missing");
    };
    root_node.layout_spec = Box::new(OffsetLayout::new(1, updated_offset, root_calls.clone()));

    let layout_ctx = LayoutContext {
        cache: &layout_cache.entries,
        frame_index: 2,
    };

    let _ = measure_root(
        root_id,
        &constraint,
        tree.tree(),
        tree.metadatas(),
        resource_manager.clone(),
        &gpu,
        Some(&layout_ctx),
    );

    assert_eq!(root_calls.load(Ordering::SeqCst), 2);
    assert_eq!(child_calls.load(Ordering::SeqCst), 1);

    let child_meta = match tree.metadatas().get(&child_id) {
        Some(meta) => meta,
        None => panic!("child metadata missing after update"),
    };
    assert_eq!(child_meta.rel_position, Some(updated_offset));
}
