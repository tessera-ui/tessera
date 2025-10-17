use std::{
    any::TypeId,
    collections::{BinaryHeap, HashMap},
};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::IntoNodeIdentifiers,
};

use crate::{
    px::{Px, PxPosition, PxRect, PxSize},
    renderer::command::{BarrierRequirement, Command},
};

/// Instruction category for sorting.
/// The order of the variants is important as it defines the priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum InstructionCategory {
    /// Low priority, can be batched together.
    ContinuationDraw,
    /// Medium priority, requires a barrier.
    BarrierDraw,
    /// High priority, must be executed before barrier draws that depend on it.
    Compute,
    /// A state-changing command that acts as a reordering fence.
    StateChange,
}

/// A wrapper for a command with additional information for sorting.
pub(crate) struct InstructionInfo {
    pub(crate) original_index: usize,
    pub(crate) command: Command,
    pub(crate) type_id: TypeId,
    pub(crate) size: PxSize,
    pub(crate) position: PxPosition,
    pub(crate) category: InstructionCategory,
    pub(crate) rect: PxRect,
}

impl InstructionInfo {
    /// Creates a new `InstructionInfo` from a command and its context.
    ///
    /// It calculates the instruction category and the bounding rectangle.
    pub(crate) fn new(
        (command, type_id, size, position): (Command, TypeId, PxSize, PxPosition),
        original_index: usize,
    ) -> Self {
        let (category, rect) = match &command {
            Command::Compute(command) => {
                // Compute commands should have proper scoping based on their barrier requirement
                // instead of always using global scope
                let barrier_req = command.barrier();
                let rect = match barrier_req {
                    BarrierRequirement::Global => PxRect {
                        x: Px(0),
                        y: Px(0),
                        width: Px(i32::MAX),
                        height: Px(i32::MAX),
                    },
                    BarrierRequirement::PaddedLocal {
                        top,
                        right,
                        bottom,
                        left,
                    } => {
                        let padded_x = (position.x - left).max(Px(0));
                        let padded_y = (position.y - top).max(Px(0));
                        let padded_width = size.width + left + right;
                        let padded_height = size.height + top + bottom;
                        PxRect {
                            x: padded_x,
                            y: padded_y,
                            width: padded_width,
                            height: padded_height,
                        }
                    }
                    BarrierRequirement::Absolute(rect) => rect,
                };
                (InstructionCategory::Compute, rect)
            }
            Command::Draw(draw_command) => {
                let barrier = draw_command.barrier();
                let category = if barrier.is_some() {
                    InstructionCategory::BarrierDraw
                } else {
                    InstructionCategory::ContinuationDraw
                };

                let rect = match barrier {
                    Some(BarrierRequirement::Global) => PxRect {
                        x: Px(0),
                        y: Px(0),
                        width: Px(i32::MAX),
                        height: Px(i32::MAX),
                    },
                    Some(BarrierRequirement::PaddedLocal {
                        top,
                        right,
                        bottom,
                        left,
                    }) => {
                        let padded_x = (position.x - left).max(Px(0));
                        let padded_y = (position.y - top).max(Px(0));
                        let padded_width = size.width + left + right;
                        let padded_height = size.height + top + bottom;
                        PxRect {
                            x: padded_x,
                            y: padded_y,
                            width: padded_width,
                            height: padded_height,
                        }
                    }
                    Some(BarrierRequirement::Absolute(rect)) => rect,
                    None => PxRect {
                        x: position.x,
                        y: position.y,
                        width: size.width,
                        height: size.height,
                    },
                };
                (category, rect)
            }
            Command::ClipPush(rect) => (InstructionCategory::StateChange, *rect),
            Command::ClipPop => (
                InstructionCategory::StateChange,
                PxRect {
                    x: position.x,
                    y: position.y,
                    width: Px::ZERO,
                    height: Px::ZERO,
                },
            ),
        };

        Self {
            original_index,
            command,
            type_id,
            size,
            position,
            category,
            rect,
        }
    }
}

/// A node in the priority queue for topological sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PriorityNode {
    category: InstructionCategory,
    type_id: TypeId,
    original_index: usize,
    node_index: NodeIndex,
    batch_potential: usize,
}

impl Ord for PriorityNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // This is the core heuristic for optimal batching:
        // 1. Higher category is always higher priority.
        // 2. For the same category, nodes with smaller batch potential are prioritized.
        //    This helps to get "lonely" nodes out of the way, clearing the path for
        //    larger batches to be processed contiguously.
        // 3. The original index is used as a final tie-breaker for stability.
        self.category
            .cmp(&other.category)
            .then_with(|| self.batch_potential.cmp(&other.batch_potential))
            .then_with(|| self.original_index.cmp(&other.original_index))
    }
}

impl PartialOrd for PriorityNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) fn reorder_instructions(
    commands: impl IntoIterator<Item = (Command, TypeId, PxSize, PxPosition)>,
) -> Vec<(Command, TypeId, PxSize, PxPosition)> {
    let instructions: Vec<InstructionInfo> = commands
        .into_iter()
        .enumerate()
        .map(|(i, cmd)| InstructionInfo::new(cmd, i))
        .collect();

    if instructions.is_empty() {
        return vec![];
    }

    let mut potentials = HashMap::new();
    for info in &instructions {
        *potentials.entry((info.category, info.type_id)).or_insert(0) += 1;
    }

    let graph = build_dependency_graph(&instructions);

    let sorted_node_indices = priority_topological_sort(&graph, &instructions, &potentials);

    let mut sorted_instructions = Vec::with_capacity(instructions.len());
    let mut original_infos: Vec<_> = instructions.into_iter().map(Some).collect();

    for node_index in sorted_node_indices {
        let original_index = node_index.index();
        if let Some(info) = original_infos[original_index].take() {
            sorted_instructions.push((info.command, info.type_id, info.size, info.position));
        }
    }

    sorted_instructions
}

fn priority_topological_sort(
    graph: &DiGraph<(), ()>,
    instructions: &[InstructionInfo],
    potentials: &HashMap<(InstructionCategory, TypeId), usize>,
) -> Vec<NodeIndex> {
    let mut in_degree = vec![0; graph.node_count()];
    for edge in graph.raw_edges() {
        in_degree[edge.target().index()] += 1;
    }

    let mut ready_queue = BinaryHeap::new();
    for node_index in graph.node_identifiers() {
        if in_degree[node_index.index()] == 0 {
            let info = &instructions[node_index.index()];
            ready_queue.push(PriorityNode {
                category: info.category,
                type_id: info.type_id,
                original_index: info.original_index,
                node_index,
                batch_potential: potentials[&(info.category, info.type_id)],
            });
        }
    }

    let mut sorted_list = Vec::with_capacity(instructions.len());
    while let Some(priority_node) = ready_queue.pop() {
        let u = priority_node.node_index;
        sorted_list.push(u);

        for v in graph.neighbors(u) {
            in_degree[v.index()] -= 1;
            if in_degree[v.index()] == 0 {
                let info = &instructions[v.index()];
                ready_queue.push(PriorityNode {
                    category: info.category,
                    type_id: info.type_id,
                    original_index: info.original_index,
                    node_index: v,
                    batch_potential: potentials[&(info.category, info.type_id)],
                });
            }
        }
    }

    if sorted_list.len() != instructions.len() {
        // This indicates a cycle in the graph, which should not happen
        // in a well-formed UI command stream.
        // Fallback to original order.
        return (0..instructions.len()).map(NodeIndex::new).collect();
    }

    sorted_list
}

fn build_dependency_graph(instructions: &[InstructionInfo]) -> DiGraph<(), ()> {
    let mut graph = DiGraph::new();
    let node_indices: Vec<NodeIndex> = instructions.iter().map(|_| graph.add_node(())).collect();

    for i in 0..instructions.len() {
        for j in 0..instructions.len() {
            if i == j {
                continue;
            }

            let inst_i = &instructions[i];
            let inst_j = &instructions[j];

            // Rule 0: State changes act as fences.
            // If one of two commands is a state change, their relative order must be preserved.
            if inst_i.original_index < inst_j.original_index
                && (inst_i.category == InstructionCategory::StateChange
                    || inst_j.category == InstructionCategory::StateChange)
            {
                graph.add_edge(node_indices[i], node_indices[j], ());
            }

            // Rule 1: Explicit dependency (Compute -> BarrierDraw)
            // If inst_j is a BarrierDraw and inst_i is a Compute that appeared
            // earlier, then j depends on i.
            if inst_i.category == InstructionCategory::Compute
                && inst_j.category == InstructionCategory::BarrierDraw
                && inst_i.original_index < inst_j.original_index
            {
                graph.add_edge(node_indices[i], node_indices[j], ());
            }

            // Rule 2: Implicit dependency (Overlapping Draws)
            // If both are draw commands and their original order matters (j came after i)
            // and their rectangles are not orthogonal (i.e., they might overlap),
            // then j depends on i to maintain painter's algorithm.
            if (inst_i.category == InstructionCategory::BarrierDraw
                || inst_i.category == InstructionCategory::ContinuationDraw)
                && (inst_j.category == InstructionCategory::BarrierDraw
                    || inst_j.category == InstructionCategory::ContinuationDraw)
                && inst_i.original_index < inst_j.original_index
                && !inst_i.rect.is_orthogonal(&inst_j.rect)
            {
                graph.add_edge(node_indices[i], node_indices[j], ());
            }

            // Rule 3: Implicit dependency (Draw -> Compute)
            // If inst_j is a Compute command and inst_i is a Draw command that
            // appeared earlier, and their areas are not orthogonal, then j depends on i.
            if (inst_i.category == InstructionCategory::BarrierDraw
                || inst_i.category == InstructionCategory::ContinuationDraw)
                && inst_j.category == InstructionCategory::Compute
                && inst_i.original_index < inst_j.original_index
                && !inst_i.rect.is_orthogonal(&inst_j.rect)
            {
                graph.add_edge(node_indices[i], node_indices[j], ());
            }
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        px::{Px, PxPosition, PxRect, PxSize},
        renderer::{
            BarrierRequirement, command::Command, compute::ComputeCommand, drawer::DrawCommand,
        },
    };
    use std::any::TypeId;
    use std::fmt::Debug;

    // --- Mock Commands ---
    // Mocks to simulate different command types for testing reordering logic.

    #[derive(Debug, PartialEq, Clone)]
    struct MockDrawCommand {
        barrier_req: Option<BarrierRequirement>,
    }

    impl DrawCommand for MockDrawCommand {
        fn barrier(&self) -> Option<BarrierRequirement> {
            self.barrier_req
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    struct MockDrawCommand2 {
        barrier_req: Option<BarrierRequirement>,
    }

    impl DrawCommand for MockDrawCommand2 {
        fn barrier(&self) -> Option<BarrierRequirement> {
            self.barrier_req
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    struct MockComputeCommand {
        barrier_req: BarrierRequirement,
    }

    impl ComputeCommand for MockComputeCommand {
        fn barrier(&self) -> BarrierRequirement {
            self.barrier_req
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    struct MockComputeCommand2 {
        barrier_req: BarrierRequirement,
    }

    impl ComputeCommand for MockComputeCommand2 {
        fn barrier(&self) -> BarrierRequirement {
            self.barrier_req
        }
    }

    // --- Helper Functions ---

    fn create_cmd(
        pos: PxPosition,
        barrier_req: Option<BarrierRequirement>,
        is_compute: bool,
    ) -> (Command, TypeId, PxSize, PxPosition) {
        let size = PxSize::new(Px(10), Px(10));
        if is_compute {
            let cmd = MockComputeCommand {
                barrier_req: barrier_req.unwrap_or(BarrierRequirement::Global),
            };
            (
                Command::Compute(Box::new(cmd)),
                TypeId::of::<MockComputeCommand>(),
                size,
                pos,
            )
        } else {
            let cmd = MockDrawCommand { barrier_req };
            (
                Command::Draw(Box::new(cmd)),
                TypeId::of::<MockDrawCommand>(),
                size,
                pos,
            )
        }
    }

    fn create_cmd2(
        pos: PxPosition,
        barrier_req: Option<BarrierRequirement>,
        is_compute: bool,
    ) -> (Command, TypeId, PxSize, PxPosition) {
        let size = PxSize::new(Px(10), Px(10));
        if is_compute {
            let cmd = MockComputeCommand2 {
                barrier_req: barrier_req.unwrap_or(BarrierRequirement::Global),
            };
            (
                Command::Compute(Box::new(cmd)),
                TypeId::of::<MockComputeCommand2>(),
                size,
                pos,
            )
        } else {
            let cmd = MockDrawCommand2 { barrier_req };
            (
                Command::Draw(Box::new(cmd)),
                TypeId::of::<MockDrawCommand2>(),
                size,
                pos,
            )
        }
    }

    fn get_positions(commands: &[(Command, TypeId, PxSize, PxPosition)]) -> Vec<PxPosition> {
        commands.iter().map(|(_, _, _, pos)| *pos).collect()
    }

    // --- Test Cases ---

    #[test]
    fn test_empty_instructions() {
        let commands = vec![];
        let reordered = reorder_instructions(commands);
        assert!(reordered.is_empty());
    }

    #[test]
    fn test_no_dependencies_preserves_order() {
        let commands = vec![
            create_cmd(PxPosition::new(Px(0), Px(0)), None, false), // 0
            create_cmd(PxPosition::new(Px(20), Px(0)), None, false), // 1
        ];
        let original_positions = get_positions(&commands);
        let reordered = reorder_instructions(commands);
        let reordered_positions = get_positions(&reordered);
        assert_eq!(reordered_positions, original_positions);
    }

    #[test]
    fn test_compute_before_barrier_preserves_order() {
        let commands = vec![
            create_cmd(
                PxPosition::new(Px(0), Px(0)),
                Some(BarrierRequirement::Global),
                true,
            ), // 0: Compute
            create_cmd(
                PxPosition::new(Px(20), Px(20)),
                Some(BarrierRequirement::Global),
                false,
            ), // 1: BarrierDraw
        ];
        let original_positions = get_positions(&commands);
        let reordered = reorder_instructions(commands);
        let reordered_positions = get_positions(&reordered);
        assert_eq!(reordered_positions, original_positions);
    }

    #[test]
    fn test_opt() {
        // Test case 1: No dependencies, test batching
        let commands = vec![
            create_cmd(PxPosition::new(Px(0), Px(0)), None, false), // 0 (T1)
            create_cmd2(PxPosition::new(Px(10), Px(10)), None, false), // 1 (T2)
            create_cmd(PxPosition::new(Px(20), Px(20)), None, false), // 2 (T1)
        ];
        let reordered = reorder_instructions(commands);
        let reordered_positions = get_positions(&reordered);

        // Potentials: T1 -> 2, T2 -> 1.
        // T2 has lower potential, so it's prioritized.
        // Expected order: [1, 0, 2]
        let expected_positions = vec![
            PxPosition::new(Px(10), Px(10)), // 1
            PxPosition::new(Px(0), Px(0)),   // 0
            PxPosition::new(Px(20), Px(20)), // 2
        ];
        assert_eq!(reordered_positions, expected_positions);

        // Test case 2: With dependencies, test batching
        let commands = vec![
            create_cmd(PxPosition::new(Px(0), Px(0)), None, false), // 0 (T1)
            create_cmd2(PxPosition::new(Px(10), Px(10)), None, false), // 1 (T2)
            create_cmd(PxPosition::new(Px(5), Px(5)), None, false), // 2 (T1)
        ];
        let reordered = reorder_instructions(commands);
        let reordered_positions = get_positions(&reordered);

        // Potentials: T1 -> 2, T2 -> 1.
        // Dependencies: 2 > 0, 2 > 1.
        // Initial ready queue: [0, 1].
        // Node 1 has lower potential (1 vs 2), so it's prioritized.
        // Expected order: [1, 0, 2]
        let expected_positions = vec![
            PxPosition::new(Px(10), Px(10)), // Cmd 1
            PxPosition::new(Px(0), Px(0)),   // Cmd 0
            PxPosition::new(Px(5), Px(5)),   // Cmd 2
        ];
        assert_eq!(expected_positions, reordered_positions);
    }

    #[test]
    fn test_overlapping_draw_preserves_order() {
        let commands = vec![
            create_cmd(PxPosition::new(Px(0), Px(0)), None, false), // 0
            create_cmd(PxPosition::new(Px(5), Px(5)), None, false), // 1 (overlaps with 0)
        ];
        let original_positions = get_positions(&commands);
        let reordered = reorder_instructions(commands);
        let reordered_positions = get_positions(&reordered);
        assert_eq!(reordered_positions, original_positions);
    }

    #[test]
    fn test_draw_before_overlapping_compute_preserves_order() {
        let commands = vec![
            create_cmd(
                PxPosition::new(Px(0), Px(0)),
                Some(BarrierRequirement::Global),
                false,
            ), // 0: BarrierDraw
            create_cmd(
                PxPosition::new(Px(20), Px(20)),
                Some(BarrierRequirement::Global),
                true,
            ), // 1: Compute
        ];
        let original_positions = get_positions(&commands);
        let reordered = reorder_instructions(commands);
        let reordered_positions = get_positions(&reordered);
        assert_eq!(reordered_positions, original_positions);
    }

    #[test]
    fn test_reorder_based_on_priority_with_no_overlap() {
        let commands = vec![
            create_cmd(
                PxPosition::new(Px(0), Px(0)),
                Some(BarrierRequirement::Absolute(PxRect::new(
                    Px(0),
                    Px(0),
                    Px(10),
                    Px(10),
                ))), // rect A
                false, // BarrierDraw
            ), // 0
            create_cmd(
                PxPosition::new(Px(100), Px(100)),
                Some(BarrierRequirement::Absolute(PxRect::new(
                    Px(100),
                    Px(100),
                    Px(10),
                    Px(10),
                ))), // rect B
                true, // Compute
            ), // 1
            create_cmd(PxPosition::new(Px(200), Px(200)), None, false), // 2: ContinuationDraw
        ];
        let original_positions = get_positions(&commands);
        // No dependencies as all rects are orthogonal.
        // Priority: Compute (1) > BarrierDraw (0) > ContinuationDraw (2)
        let reordered = reorder_instructions(commands);
        let reordered_positions = get_positions(&reordered);

        let expected_positions = vec![
            original_positions[1], // Compute
            original_positions[0], // BarrierDraw
            original_positions[2], // ContinuationDraw
        ];
        assert_eq!(reordered_positions, expected_positions);
    }

    #[test]
    fn test_complex_reordering_with_dependencies() {
        let commands = vec![
            // 0: Compute. Must run first.
            create_cmd(
                PxPosition::new(Px(0), Px(0)),
                Some(BarrierRequirement::Global),
                true,
            ),
            // 1: BarrierDraw. Depends on 0. Orthogonal to 4.
            create_cmd(
                PxPosition::new(Px(50), Px(50)),
                Some(BarrierRequirement::Absolute(PxRect::new(
                    Px(50),
                    Px(50),
                    Px(10),
                    Px(10),
                ))),
                false,
            ),
            // 2: ContinuationDraw. Overlaps with 3.
            create_cmd(PxPosition::new(Px(200), Px(200)), None, false),
            // 3: ContinuationDraw.
            create_cmd(PxPosition::new(Px(205), Px(205)), None, false),
            // 4: BarrierDraw. Depends on 0. Orthogonal to 1.
            create_cmd(
                PxPosition::new(Px(80), Px(80)),
                Some(BarrierRequirement::Absolute(PxRect::new(
                    Px(80),
                    Px(80),
                    Px(10),
                    Px(10),
                ))),
                false,
            ),
        ];
        let original_positions = get_positions(&commands);

        // Dependencies:
        // 0 -> 1 (Compute -> Barrier)
        // 0 -> 4 (Compute -> Barrier)
        // 2 -> 3 (Overlapping Draw)
        // Potentials: Compute:1, BarrierDraw:2, ContinuationDraw:2
        // All categories have different potentials, so batching heuristic won't apply across categories.
        // Ready queue starts with [0(C), 2(CD)] -> Prio sort -> [0, 2]
        // 1. Pop 0. Result: [0]. Add 1, 4 to queue. Queue: [1(BD), 4(BD), 2(CD)]. Prio sort: [1,4,2]
        // 2. Pop 1. Result: [0, 1].
        // 3. Pop 4. Result: [0, 1, 4].
        // 4. Pop 2. Result: [0, 1, 4, 2]. Add 3 to queue. Queue: [3]
        // 5. Pop 3. Result: [0, 1, 4, 2, 3].
        let reordered = reorder_instructions(commands);
        let reordered_positions = get_positions(&reordered);
        let expected_positions = vec![
            original_positions[0],
            original_positions[1],
            original_positions[4],
            original_positions[2],
            original_positions[3],
        ];
        assert_eq!(reordered_positions, expected_positions);
    }
}
