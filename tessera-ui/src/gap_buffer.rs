use smallvec::SmallVec;

pub(crate) struct GapBuffer<T> {
    slots: SmallVec<[Option<T>; 4]>,
    gap_start: usize,
    gap_end: usize,
}

impl<T> Default for GapBuffer<T> {
    fn default() -> Self {
        Self {
            slots: SmallVec::new(),
            gap_start: 0,
            gap_end: 0,
        }
    }
}

impl<T> GapBuffer<T> {
    pub(crate) fn len(&self) -> usize {
        self.slots.len() - (self.gap_end - self.gap_start)
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        let physical = if index < self.gap_start {
            index
        } else {
            index + self.gap_end - self.gap_start
        };
        self.slots[physical].as_ref()
    }

    pub(crate) fn position_from(
        &self,
        start: usize,
        mut predicate: impl FnMut(&T) -> bool,
    ) -> Option<usize> {
        self.iter()
            .enumerate()
            .skip(start)
            .find_map(|(index, value)| predicate(value).then_some(index))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.slots[..self.gap_start]
            .iter()
            .chain(self.slots[self.gap_end..].iter())
            .filter_map(Option::as_ref)
    }

    pub(crate) fn move_gap(&mut self, index: usize) {
        assert!(index <= self.len(), "gap index out of range");
        while self.gap_start > index {
            self.gap_start -= 1;
            self.gap_end -= 1;
            self.slots[self.gap_end] = self.slots[self.gap_start].take();
        }
        while self.gap_start < index {
            self.slots[self.gap_start] = self.slots[self.gap_end].take();
            self.gap_start += 1;
            self.gap_end += 1;
        }
    }

    pub(crate) fn insert_at_gap(&mut self, value: T) {
        self.ensure_gap();
        self.slots[self.gap_start] = Some(value);
        self.gap_start += 1;
    }

    pub(crate) fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        self.move_gap(index);
        let value = self.slots[self.gap_end].take();
        self.gap_end += 1;
        value
    }

    pub(crate) fn truncate_right(&mut self) {
        self.slots.truncate(self.gap_end);
        self.gap_end = self.slots.len();
    }

    fn ensure_gap(&mut self) {
        if self.gap_start < self.gap_end {
            return;
        }
        let old_len = self.slots.len();
        self.move_gap(self.len());
        let additional = old_len.max(4);
        self.slots.resize_with(old_len + additional, || None);
        self.gap_end += additional;
    }
}

#[cfg(test)]
mod tests {
    use super::GapBuffer;

    fn collect(buffer: &GapBuffer<usize>) -> Vec<usize> {
        buffer.iter().copied().collect()
    }

    #[test]
    fn moving_the_gap_preserves_logical_order_and_local_edits() {
        let mut buffer = GapBuffer::default();
        for value in 0..12 {
            buffer.insert_at_gap(value);
        }
        buffer.move_gap(4);
        assert_eq!(buffer.get(4), Some(&4));
        assert_eq!(collect(&buffer), (0..12).collect::<Vec<_>>());

        assert_eq!(buffer.remove(8), Some(8));
        assert_eq!(collect(&buffer), vec![0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11]);

        buffer.move_gap(4);
        buffer.insert_at_gap(99);
        assert_eq!(
            collect(&buffer),
            vec![0, 1, 2, 3, 99, 4, 5, 6, 7, 9, 10, 11]
        );
    }

    #[test]
    fn truncate_right_keeps_the_current_epoch_prefix() {
        let mut buffer = GapBuffer::default();
        for value in 0..8 {
            buffer.insert_at_gap(value);
        }
        buffer.move_gap(3);
        buffer.truncate_right();
        assert_eq!(buffer.len(), 3);
        assert_eq!(collect(&buffer), vec![0, 1, 2]);
        buffer.move_gap(0);
        assert_eq!(collect(&buffer), vec![0, 1, 2]);
    }
}
