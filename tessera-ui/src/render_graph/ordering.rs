use std::{
    any::TypeId,
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap},
    ops::{ControlFlow, Range},
};

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{px::PxRect, render::Command};

use super::{OpCategory, OpInfo, RenderGraphOp, is_state_change};

pub(super) fn order_indices(ops: &[RenderGraphOp], infos: &[OpInfo]) -> Vec<usize> {
    let mut sequence: Vec<_> = (0..ops.len()).collect();
    sequence.sort_by_key(|&index| infos[index].sequence_index);

    let mut depth = 0;
    for &index in &sequence {
        match ops[index].command {
            Command::ClipPush(_) | Command::ShapeClipPush(_) => depth += 1,
            Command::ClipPop if depth == 0 => return sequence,
            Command::ClipPop => depth -= 1,
            _ => {}
        }
    }
    if depth != 0 {
        return sequence;
    }

    let mut ready = ReadyQueue::new(infos);
    let mut ordered = Vec::with_capacity(sequence.len());
    let mut start = 0;
    // Clip changes order everything before and after them. Splitting at the boundary
    // enforces that ordering without storing an edge from every draw to every clip.
    for (position, &index) in sequence.iter().enumerate() {
        if is_state_change(&ops[index]) {
            order_span(&sequence[start..position], infos, &mut ready, &mut ordered);
            ordered.push(index);
            start = position + 1;
        }
    }
    order_span(&sequence[start..], infos, &mut ready, &mut ordered);
    ordered
}

fn order_span(
    members: &[usize],
    infos: &[OpInfo],
    ready: &mut ReadyQueue,
    ordered: &mut Vec<usize>,
) {
    if members.len() < 2 {
        ordered.extend_from_slice(members);
        return;
    }

    let Some(dependencies) = Dependencies::new(members, infos) else {
        ordered.extend_from_slice(members);
        return;
    };
    let mut in_degree = dependencies.in_degree;
    for (position, &degree) in in_degree.iter().enumerate() {
        if degree == 0 {
            ready.push(members[position], position);
        }
    }

    let mut previous = None;
    while let Some((position, type_id)) = ready.pop(previous) {
        previous = Some(type_id);
        ordered.push(members[position]);
        for &next in &dependencies.outgoing[position] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                ready.push(members[next], next);
            }
        }
    }
}

struct Dependencies {
    outgoing: Vec<SmallVec<[usize; 4]>>,
    in_degree: Vec<usize>,
}

impl Dependencies {
    fn new(members: &[usize], infos: &[OpInfo]) -> Option<Self> {
        let mut outgoing = vec![SmallVec::new(); members.len()];
        let mut in_degree = vec![0; members.len()];
        let index = BoundsIndex::new(members.iter().enumerate().filter_map(|(position, &i)| {
            let bounds = OpBounds::new(&infos[i]).envelope()?;
            Some(Entry { position, bounds })
        }));
        // Dense overlap can defeat spatial pruning. Painter order is already correct,
        // so bound the work and edges spent looking for opportunities to batch.
        let depth = members.len().ilog2() as usize + 1;
        let mut checks = members.len().saturating_mul(depth * BoundsIndex::LEAF_SIZE);
        for (left, &operation) in members.iter().enumerate() {
            let bounds = OpBounds::new(&infos[operation]);
            if let Some(envelope) = bounds.envelope()
                && index
                    .visit_after(envelope, left, &mut checks, &mut |right| {
                        if bounds.conflicts(OpBounds::new(&infos[members[right]])) {
                            outgoing[left].push(right);
                            in_degree[right] += 1;
                        }
                    })
                    .is_break()
            {
                return None;
            }
        }
        Some(Self {
            outgoing,
            in_degree,
        })
    }
}

type BatchKey = (OpCategory, TypeId);
type Head = (OpCategory, Reverse<usize>, Reverse<usize>, usize);

struct Batch {
    key: BatchKey,
    total: usize,
    positions: BinaryHeap<Reverse<usize>>,
}

impl Batch {
    fn head(&self, id: usize) -> Option<Head> {
        self.positions
            .peek()
            .map(|&position| (self.key.0, Reverse(self.total), position, id))
    }
}

struct ReadyQueue {
    by_key: FxHashMap<BatchKey, usize>,
    for_operation: Vec<usize>,
    batches: Vec<Batch>,
    heads: BTreeSet<Head>,
}

impl ReadyQueue {
    fn new(infos: &[OpInfo]) -> Self {
        let mut by_key = FxHashMap::default();
        let mut batches = Vec::new();
        let for_operation = infos
            .iter()
            .map(|info| {
                let key = (info.category, info.type_id);
                let id = *by_key.entry(key).or_insert_with(|| {
                    let id = batches.len();
                    batches.push(Batch {
                        key,
                        total: 0,
                        positions: BinaryHeap::new(),
                    });
                    id
                });
                batches[id].total += 1;
                id
            })
            .collect();
        Self {
            by_key,
            for_operation,
            batches,
            heads: BTreeSet::new(),
        }
    }

    fn push(&mut self, operation: usize, position: usize) {
        let id = self.for_operation[operation];
        let batch = &mut self.batches[id];
        if let Some(head) = batch.head(id) {
            self.heads.remove(&head);
        }
        batch.positions.push(Reverse(position));
        self.heads
            .insert(batch.head(id).expect("batch received a ready operation"));
    }

    fn pop(&mut self, previous: Option<TypeId>) -> Option<(usize, TypeId)> {
        let &(category, _, _, best) = self.heads.last()?;
        let id = previous
            .and_then(|type_id| self.by_key.get(&(category, type_id)).copied())
            .filter(|&id| !self.batches[id].positions.is_empty())
            .unwrap_or(best);
        let batch = &mut self.batches[id];
        self.heads
            .remove(&batch.head(id).expect("selected batch is ready"));
        let Reverse(position) = batch.positions.pop().expect("selected batch is ready");
        if let Some(head) = batch.head(id) {
            self.heads.insert(head);
        }
        Some((position, batch.key.1))
    }
}

#[derive(Clone, Copy)]
struct Bounds {
    start: [i64; 2],
    end: [i64; 2],
}

impl From<PxRect> for Bounds {
    fn from(rect: PxRect) -> Self {
        let x = i64::from(rect.x.0);
        let y = i64::from(rect.y.0);
        Self {
            start: [x, y],
            end: [x + i64::from(rect.width.0), y + i64::from(rect.height.0)],
        }
    }
}

impl Bounds {
    fn envelope(self) -> Self {
        Self {
            start: [
                self.start[0].min(self.end[0]),
                self.start[1].min(self.end[1]),
            ],
            end: [
                self.start[0].max(self.end[0]),
                self.start[1].max(self.end[1]),
            ],
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            start: [
                self.start[0].min(other.start[0]),
                self.start[1].min(other.start[1]),
            ],
            end: [self.end[0].max(other.end[0]), self.end[1].max(other.end[1])],
        }
    }

    fn overlaps(self, other: Self) -> bool {
        self.start[0] < other.end[0]
            && other.start[0] < self.end[0]
            && self.start[1] < other.end[1]
            && other.start[1] < self.end[1]
    }

    fn center_sum(self, axis: usize) -> i64 {
        self.start[axis] + self.end[axis]
    }
}

struct OpBounds {
    read: Option<Bounds>,
    write: Option<Bounds>,
}

impl OpBounds {
    fn new(info: &OpInfo) -> Self {
        Self {
            read: info.read_rect.map(Bounds::from),
            write: info.write_rect.map(Bounds::from),
        }
    }

    fn envelope(&self) -> Option<Bounds> {
        match (self.read, self.write) {
            (Some(read), Some(write)) => Some(read.envelope().union(write.envelope())),
            (Some(bounds), None) | (None, Some(bounds)) => Some(bounds.envelope()),
            (None, None) => None,
        }
    }

    fn conflicts(&self, other: Self) -> bool {
        fn overlap(a: Option<Bounds>, b: Option<Bounds>) -> bool {
            matches!((a, b), (Some(a), Some(b)) if a.overlaps(b))
        }
        overlap(self.write, other.write)
            || overlap(self.write, other.read)
            || overlap(self.read, other.write)
    }
}

struct Entry {
    position: usize,
    bounds: Bounds,
}

enum Children {
    Entries(Range<usize>),
    Branch(usize, usize),
}

struct Node {
    bounds: Bounds,
    last_position: usize,
    children: Children,
}

struct BoundsIndex {
    entries: Vec<Entry>,
    nodes: Vec<Node>,
}

impl BoundsIndex {
    const LEAF_SIZE: usize = 8;

    fn new(entries: impl Iterator<Item = Entry>) -> Self {
        let mut entries: Vec<_> = entries.collect();
        let mut nodes = Vec::new();
        if !entries.is_empty() {
            Self::build(&mut entries, 0, &mut nodes);
        }
        Self { entries, nodes }
    }

    fn build(entries: &mut [Entry], offset: usize, nodes: &mut Vec<Node>) -> usize {
        let mut bounds = entries[0].bounds;
        let mut last_position = 0;
        let mut min_center = [i64::MAX; 2];
        let mut max_center = [i64::MIN; 2];
        for entry in entries.iter() {
            bounds = bounds.union(entry.bounds);
            last_position = last_position.max(entry.position);
            for axis in 0..2 {
                let center = entry.bounds.center_sum(axis);
                min_center[axis] = min_center[axis].min(center);
                max_center[axis] = max_center[axis].max(center);
            }
        }

        let children = if entries.len() <= Self::LEAF_SIZE {
            Children::Entries(offset..offset + entries.len())
        } else {
            let axis = usize::from(max_center[1] - min_center[1] > max_center[0] - min_center[0]);
            let middle = entries.len() / 2;
            entries.select_nth_unstable_by_key(middle, |entry| entry.bounds.center_sum(axis));
            let (left, right) = entries.split_at_mut(middle);
            Children::Branch(
                Self::build(left, offset, nodes),
                Self::build(right, offset + middle, nodes),
            )
        };
        let index = nodes.len();
        nodes.push(Node {
            bounds,
            last_position,
            children,
        });
        index
    }

    fn visit_after(
        &self,
        bounds: Bounds,
        position: usize,
        checks: &mut usize,
        visit: &mut impl FnMut(usize),
    ) -> ControlFlow<()> {
        if let Some(root) = self.nodes.len().checked_sub(1) {
            self.visit(root, bounds, position, checks, visit)?;
        }
        ControlFlow::Continue(())
    }

    fn visit(
        &self,
        index: usize,
        bounds: Bounds,
        position: usize,
        checks: &mut usize,
        visit: &mut impl FnMut(usize),
    ) -> ControlFlow<()> {
        let Some(remaining) = checks.checked_sub(1) else {
            return ControlFlow::Break(());
        };
        *checks = remaining;
        let node = &self.nodes[index];
        if node.last_position <= position || !node.bounds.overlaps(bounds) {
            return ControlFlow::Continue(());
        }
        match &node.children {
            Children::Entries(range) => {
                let Some(remaining) = checks.checked_sub(range.len()) else {
                    return ControlFlow::Break(());
                };
                *checks = remaining;
                for entry in &self.entries[range.clone()] {
                    if entry.position > position && entry.bounds.overlaps(bounds) {
                        visit(entry.position);
                    }
                }
            }
            &Children::Branch(left, right) => {
                self.visit(left, bounds, position, checks, visit)?;
                self.visit(right, bounds, position, checks, visit)?;
            }
        }
        ControlFlow::Continue(())
    }
}
