use std::{mem, ops::Deref, ptr::NonNull, rc::Rc};

use bumpalo::Bump;
use smallvec::SmallVec;

use super::{ActionKinds, ModifierAction};

pub(super) struct NodeRef<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> NodeRef<T> {
    pub(super) fn as_ref(&self) -> &T {
        // SAFETY: references never leave their owning chain's borrow. Each
        // pointer is registered in the fragment that owns the allocation.
        unsafe { self.0.as_ref() }
    }

    pub(super) fn ptr_eq(lhs: &Self, rhs: &Self) -> bool {
        std::ptr::addr_eq(lhs.0.as_ptr(), rhs.0.as_ptr())
    }

    pub(super) fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}

impl<T: ?Sized> Deref for NodeRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

struct Destructor {
    pointer: NonNull<()>,
    drop_value: unsafe fn(NonNull<()>),
}

impl Drop for Destructor {
    fn drop(&mut self) {
        // SAFETY: the registered value is initialized, uniquely owned by this
        // record, and its arena is destroyed only after all records are dropped.
        unsafe { (self.drop_value)(self.pointer) };
    }
}

enum Entry {
    Action(ModifierAction),
    Chain(Chain),
}

#[derive(Default)]
struct Fragment {
    entries: SmallVec<[Entry; 4]>,
    destructors: SmallVec<[Destructor; 4]>,
    arena: Bump,
    action_len: usize,
}

impl Fragment {
    fn allocate<N: 'static, T: ?Sized>(&mut self, node: N, erase: fn(&N) -> &T) -> NodeRef<T> {
        // Zero-sized values still need distinct retained identities.
        let value = if mem::size_of::<N>() == 0 {
            &mut self.arena.alloc((0u8, node)).1
        } else {
            self.arena.alloc(node)
        };
        let pointer = NonNull::from(&mut *value);
        if mem::needs_drop::<N>() {
            unsafe fn drop_value<N>(pointer: NonNull<()>) {
                // SAFETY: this record was registered for precisely this N.
                unsafe { pointer.cast::<N>().as_ptr().drop_in_place() };
            }
            self.destructors.push(Destructor {
                pointer: pointer.cast(),
                drop_value: drop_value::<N>,
            });
        }
        NodeRef(NonNull::from(erase(value)))
    }
}

pub(crate) struct OwnedNode<T: ?Sized> {
    pointer: NodeRef<T>,
    owner: Rc<Fragment>,
}

impl<T: ?Sized> OwnedNode<T> {
    pub(crate) fn new<N: 'static>(node: N, erase: fn(&N) -> &T) -> Self {
        let mut owner = Fragment::default();
        let pointer = owner.allocate(node, erase);
        Self {
            pointer,
            owner: Rc::new(owner),
        }
    }
}

impl<T: ?Sized> Clone for OwnedNode<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: NodeRef(self.pointer.0),
            owner: self.owner.clone(),
        }
    }
}

impl<T: ?Sized> Deref for OwnedNode<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.pointer.as_ref()
    }
}

impl<T: ?Sized> AsRef<T> for OwnedNode<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl Drop for Fragment {
    fn drop(&mut self) {
        let mut pending = SmallVec::<[Chain; 4]>::new();
        let collect = |entries: SmallVec<[Entry; 4]>, pending: &mut SmallVec<[Chain; 4]>| {
            for entry in entries {
                if let Entry::Chain(chain) = entry {
                    pending.push(chain);
                }
            }
        };
        collect(mem::take(&mut self.entries), &mut pending);
        while let Some(mut chain) = pending.pop() {
            if let Some(root) = chain.root.take()
                && let Ok(mut fragment) = Rc::try_unwrap(root)
            {
                collect(mem::take(&mut fragment.entries), &mut pending);
            }
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct Chain {
    root: Option<Rc<Fragment>>,
    start: usize,
    len: usize,
    kinds: ActionKinds,
}

impl Chain {
    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(super) fn is_same(&self, other: &Self) -> bool {
        self.start == other.start
            && self.len == other.len
            && match (&self.root, &other.root) {
                (None, None) => true,
                (Some(lhs), Some(rhs)) => Rc::ptr_eq(lhs, rhs),
                _ => false,
            }
    }

    pub(super) fn kinds(&self) -> ActionKinds {
        self.kinds
    }

    fn writable(&mut self) -> &mut Fragment {
        let unique = self.start == 0
            && self
                .root
                .as_mut()
                .is_some_and(|root| self.len == root.action_len && Rc::get_mut(root).is_some());
        if !unique {
            let previous = mem::take(self);
            self.len = previous.len;
            self.kinds = previous.kinds;
            let mut fragment = Fragment::default();
            if !previous.is_empty() {
                fragment.entries.push(Entry::Chain(previous));
            }
            fragment.action_len = self.len;
            self.root = Some(Rc::new(fragment));
        }
        Rc::get_mut(self.root.as_mut().expect("writable fragment")).expect("unique fragment")
    }

    pub(super) fn push(mut self, action: ModifierAction) -> Self {
        let kinds = self.kinds.union(action.kind());
        let fragment = self.writable();
        fragment.entries.push(Entry::Action(action));
        fragment.action_len += 1;
        self.len += 1;
        self.kinds = kinds;
        self
    }

    pub(super) fn push_node<N: 'static, T: ?Sized>(
        mut self,
        node: N,
        erase: fn(&N) -> &T,
        action: fn(NodeRef<T>) -> ModifierAction,
    ) -> Self {
        let erased = self.writable().allocate(node, erase);
        let action = action(erased);
        let kinds = self.kinds.union(action.kind());
        let fragment = self.writable();
        fragment.entries.push(Entry::Action(action));
        fragment.action_len += 1;
        self.len += 1;
        self.kinds = kinds;
        self
    }

    pub(super) fn then(mut self, other: Self) -> Self {
        if other.is_empty() {
            return self;
        }
        if self.is_empty() {
            return other;
        }
        let added = other.len;
        let len = self.len + added;
        let kinds = self.kinds.union(other.kinds);
        let fragment = self.writable();
        fragment.entries.push(Entry::Chain(other));
        fragment.action_len += added;
        self.len = len;
        self.kinds = kinds;
        self
    }

    pub(super) fn slice(&self, start: usize, len: usize) -> Self {
        assert!(start <= self.len && len <= self.len - start);
        if len == 0 {
            return Self::default();
        }
        let mut kinds = ActionKinds::default();
        for action in self.actions().skip(start).take(len) {
            kinds = kinds.union(action.kind());
        }
        Self {
            root: self.root.clone(),
            start: self.start + start,
            len,
            kinds,
        }
    }

    pub(super) fn actions(&self) -> impl Iterator<Item = &ModifierAction> {
        struct Range<'a> {
            entries: std::slice::Iter<'a, Entry>,
            skip: usize,
            remaining: usize,
        }
        let mut stack = SmallVec::<[Range<'_>; 4]>::new();
        if let Some(root) = &self.root {
            stack.push(Range {
                entries: root.entries.iter(),
                skip: self.start,
                remaining: self.len,
            });
        }
        std::iter::from_fn(move || {
            while let Some(range) = stack.last_mut() {
                if range.remaining == 0 {
                    stack.pop();
                    continue;
                }
                match range.entries.next() {
                    None => {
                        stack.pop();
                    }
                    Some(Entry::Action(action)) => {
                        if range.skip > 0 {
                            range.skip -= 1;
                            continue;
                        }
                        range.remaining -= 1;
                        return Some(action);
                    }
                    Some(Entry::Chain(chain)) => {
                        if range.skip >= chain.len {
                            range.skip -= chain.len;
                            continue;
                        }
                        let skip = mem::take(&mut range.skip);
                        let len = range.remaining.min(chain.len - skip);
                        range.remaining -= len;
                        if let Some(root) = &chain.root {
                            stack.push(Range {
                                entries: root.entries.iter(),
                                skip: chain.start + skip,
                                remaining: len,
                            });
                        }
                    }
                }
            }
            None
        })
    }

    pub(super) fn fold_actions<'a>(&'a self, visit: &mut impl FnMut(&'a ModifierAction)) {
        for action in self.actions() {
            visit(action);
        }
    }

    pub(super) fn for_each_action_outer_first(&self, visit: &mut impl FnMut(&ModifierAction)) {
        let actions: SmallVec<[&ModifierAction; 8]> = self.actions().collect();
        for action in actions.into_iter().rev() {
            visit(action);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use crate::modifier::{
        DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier, ModifierAction,
    };

    #[derive(PartialEq)]
    struct Width(u32);

    impl crate::modifier::LayoutModifierNode for Width {}

    #[repr(align(64))]
    struct LargeNode {
        bytes: [u8; 8192],
        drops: Rc<Cell<usize>>,
    }

    impl DrawModifierNode for LargeNode {
        fn diagnostic_name(&self) -> &'static str {
            assert_eq!((self.bytes[0], self.bytes[8191]), (37, 37));
            "large user node"
        }

        fn draw(&self, _: &mut DrawModifierContext<'_, '_>, _: &mut dyn DrawModifierContent) {}
    }

    impl Drop for LargeNode {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }

    fn large(drops: &Rc<Cell<usize>>) -> LargeNode {
        LargeNode {
            bytes: [37; 8192],
            drops: drops.clone(),
        }
    }

    #[test]
    fn heterogeneous_arena_keeps_large_aligned_non_clone_values_stable_until_last_owner() {
        let drops = Rc::new(Cell::new(0));
        let mut chain = Modifier::new().push_draw(large(&drops));
        let first = match chain.element.actions().next().unwrap() {
            ModifierAction::Draw(node) => node.as_ptr(),
            _ => unreachable!(),
        };
        assert_eq!((first as *const () as usize) % 64, 0);
        for _ in 0..64 {
            chain = chain.push_draw(large(&drops));
        }
        assert_eq!(chain.element.len(), 65);
        match chain.element.actions().next().unwrap() {
            ModifierAction::Draw(node) => {
                assert!(std::ptr::addr_eq(first, node.as_ptr()));
                assert_eq!(node.diagnostic_name(), "large user node");
            }
            _ => unreachable!(),
        }
        let shared = chain.clone();
        drop(chain);
        assert_eq!(drops.get(), 0);
        drop(shared);
        assert_eq!(drops.get(), 65);
    }

    #[test]
    fn shared_base_does_not_keep_discarded_branch_allocations_alive() {
        let base_drops = Rc::new(Cell::new(0));
        let branch_drops = Rc::new(Cell::new(0));
        let base = Modifier::new().push_draw(large(&base_drops));
        for _ in 0..128 {
            let branch = base.clone().push_draw(large(&branch_drops));
            drop(branch);
        }
        assert_eq!(branch_drops.get(), 128);
        assert_eq!(base_drops.get(), 0);
        drop(base);
        assert_eq!(base_drops.get(), 1);
    }

    #[test]
    fn appending_to_a_unique_slice_does_not_restore_its_discarded_suffix() {
        let original = Modifier::new().push_layout(Width(1)).push_layout(Width(2));
        let slice = Modifier {
            element: original.element.slice(0, 1),
        };
        drop(original);
        let extended = slice.push_layout(Width(3));
        let expected = Modifier::new().push_layout(Width(1)).push_layout(Width(3));
        assert_eq!(extended, expected);
    }

    #[test]
    fn deep_shared_descriptions_iterate_and_release_without_recursive_chain_drops() {
        let mut chain = Modifier::new();
        for index in 0..20_000 {
            chain = chain
                .clone()
                .then(Modifier::new().push_layout(Width(index)));
        }
        assert_eq!(chain.element.actions().count(), 20_000);
        drop(chain);
    }
}
