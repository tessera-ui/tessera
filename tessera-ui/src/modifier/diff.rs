use smallvec::SmallVec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Edit {
    Remove(usize),
    Insert(usize),
    Same(usize, usize),
}

#[derive(Clone, Copy)]
struct Range {
    x: isize,
    end_x: isize,
    y: isize,
    end_y: isize,
}

#[derive(Clone, Copy)]
struct Snake {
    range: Range,
    reverse: bool,
}

#[derive(Clone, Copy)]
struct Diagonal {
    x: usize,
    y: usize,
    len: usize,
}

#[derive(Default)]
pub(super) struct Workspace {
    forward: Vec<isize>,
    backward: Vec<isize>,
    ranges: SmallVec<[Range; 8]>,
    diagonals: SmallVec<[Diagonal; 8]>,
    edits: SmallVec<[Edit; 8]>,
}

impl Workspace {
    #[cfg(test)]
    pub(super) fn has_search_storage(&self) -> bool {
        !self.forward.is_empty() || !self.backward.is_empty()
    }

    pub(super) fn linear(&mut self, len: usize) -> &[Edit] {
        self.edits.clear();
        self.edits
            .extend((0..len).map(|index| Edit::Same(index, index)));
        &self.edits
    }

    pub(super) fn calculate(
        &mut self,
        old: usize,
        new: usize,
        same: impl Fn(usize, usize) -> bool,
    ) -> &[Edit] {
        self.edits.clear();
        self.ranges.clear();
        self.diagonals.clear();
        let sum = old.checked_add(new).expect("modifier diff size overflow");
        let center = isize::try_from(sum.div_ceil(2)).expect("modifier diff size overflow");
        self.forward.resize(sum + 3, 0);
        self.backward.resize(sum + 3, 0);
        self.ranges.push(Range {
            x: 0,
            end_x: old as isize,
            y: 0,
            end_y: new as isize,
        });
        while let Some(range) = self.ranges.pop() {
            if let Some(snake) =
                midpoint(range, center, &mut self.forward, &mut self.backward, &same)
            {
                let r = snake.range;
                let dx = r.end_x - r.x;
                let dy = r.end_y - r.y;
                let len = dx.min(dy);
                if len > 0 {
                    self.diagonals.push(Diagonal {
                        x: (r.x + isize::from(!snake.reverse && dx > dy)) as usize,
                        y: (r.y + isize::from(!snake.reverse && dy > dx)) as usize,
                        len: len as usize,
                    });
                }
                self.ranges.push(Range {
                    end_x: r.x,
                    end_y: r.y,
                    ..range
                });
                self.ranges.push(Range {
                    x: r.end_x,
                    y: r.end_y,
                    ..range
                });
            }
        }
        self.diagonals.sort_unstable_by_key(|d| (d.x, d.y));
        self.diagonals.push(Diagonal {
            x: old,
            y: new,
            len: 0,
        });
        let (mut x, mut y) = (0, 0);
        for diagonal in &self.diagonals {
            while x < diagonal.x {
                self.edits.push(Edit::Remove(x));
                x += 1;
            }
            while y < diagonal.y {
                self.edits.push(Edit::Insert(y));
                y += 1;
            }
            for _ in 0..diagonal.len {
                self.edits.push(Edit::Same(x, y));
                x += 1;
                y += 1;
            }
        }
        &self.edits
    }
}

fn midpoint(
    r: Range,
    center: isize,
    forward: &mut [isize],
    backward: &mut [isize],
    same: &impl Fn(usize, usize) -> bool,
) -> Option<Snake> {
    let old = r.end_x - r.x;
    let new = r.end_y - r.y;
    if old == 0 || new == 0 {
        return None;
    }
    let index = |k: isize| (center + k) as usize;
    forward[index(1)] = r.x;
    backward[index(1)] = r.end_x;
    let delta = old - new;
    for d in 0..(old + new + 1) / 2 {
        for k in (-d..=d).step_by(2) {
            let (start_x, mut x) =
                if k == -d || k != d && forward[index(k + 1)] > forward[index(k - 1)] {
                    (forward[index(k + 1)], forward[index(k + 1)])
                } else {
                    (forward[index(k - 1)], forward[index(k - 1)] + 1)
                };
            let mut y = r.y + (x - r.x) - k;
            let start_y = y - isize::from(d != 0 && x == start_x);
            while x < r.end_x && y < r.end_y && same(x as usize, y as usize) {
                x += 1;
                y += 1;
            }
            forward[index(k)] = x;
            let opposite = delta - k;
            if delta.abs() % 2 == 1
                && opposite >= -d + 1
                && opposite < d
                && backward[index(opposite)] <= x
            {
                return Some(Snake {
                    range: Range {
                        x: start_x,
                        y: start_y,
                        end_x: x,
                        end_y: y,
                    },
                    reverse: false,
                });
            }
        }
        for k in (-d..=d).step_by(2) {
            let (start_x, mut x) =
                if k == -d || k != d && backward[index(k + 1)] < backward[index(k - 1)] {
                    (backward[index(k + 1)], backward[index(k + 1)])
                } else {
                    (backward[index(k - 1)], backward[index(k - 1)] - 1)
                };
            let mut y = r.end_y - (r.end_x - x - k);
            let start_y = y + isize::from(d != 0 && x == start_x);
            while x > r.x && y > r.y && same((x - 1) as usize, (y - 1) as usize) {
                x -= 1;
                y -= 1;
            }
            backward[index(k)] = x;
            let opposite = delta - k;
            if delta % 2 == 0 && opposite >= -d && opposite <= d && forward[index(opposite)] >= x {
                return Some(Snake {
                    range: Range {
                        x,
                        y,
                        end_x: start_x,
                        end_y: start_y,
                    },
                    reverse: true,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Edit, Workspace};

    #[test]
    fn scripts_are_minimal_and_reconstruct_all_short_binary_sequences() {
        let mut workspace = Workspace::default();
        let mut sequences = Vec::new();
        for len in 0..=5 {
            for bits in 0..1usize << len {
                sequences.push((0..len).map(|i| (bits >> i) & 1).collect::<Vec<_>>());
            }
        }
        for old in &sequences {
            for new in &sequences {
                let edits = workspace.calculate(old.len(), new.len(), |x, y| old[x] == new[y]);
                let (mut x, mut y, mut cost) = (0, 0, 0);
                let mut result = Vec::new();
                for &edit in edits {
                    match edit {
                        Edit::Remove(index) => {
                            assert_eq!(index, x);
                            x += 1;
                            cost += 1;
                        }
                        Edit::Insert(index) => {
                            assert_eq!(index, y);
                            result.push(new[y]);
                            y += 1;
                            cost += 1;
                        }
                        Edit::Same(a, b) => {
                            assert_eq!((a, b), (x, y));
                            assert_eq!(old[a], new[b]);
                            result.push(old[a]);
                            x += 1;
                            y += 1;
                        }
                    }
                }
                assert_eq!((x, y), (old.len(), new.len()));
                assert_eq!(&result, new);
                let mut distances = (0..=new.len()).collect::<Vec<_>>();
                for (i, a) in old.iter().enumerate() {
                    let mut previous = distances[0];
                    distances[0] = i + 1;
                    for (j, b) in new.iter().enumerate() {
                        let diagonal = previous;
                        previous = distances[j + 1];
                        distances[j + 1] = if a == b {
                            diagonal
                        } else {
                            1 + previous.min(distances[j])
                        };
                    }
                }
                assert_eq!(cost, distances[new.len()], "{old:?} -> {new:?}");
            }
        }
    }
}
