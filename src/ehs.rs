#[path = "arith.rs"]
pub mod arith;
use self::arith::Arith;
use std::cmp::min;
use std::ops::{Range, RangeInclusive};

#[derive(Debug, PartialEq, Eq, Hash)]
struct Subset<T, M> {
    sum: T,
    mask: M,
}
impl<T: Arith> Subset<T, u64> {
    fn new(mask: u64, elements: &[T]) -> Self {
        let mut selected_bit = 1;
        let mut sum = T::from(0);
        for x in elements {
            if mask & selected_bit > 0 {
                sum += *x;
            }
            selected_bit <<= 1;
        }
        Subset { sum, mask }
    }
}

fn all_subsets<T: Arith>(elements: &[T]) -> Option<(Vec<Subset<T, u64>>)> {
    if elements.len() > 63 {
        //TODO: 64 is doable but requires care on the bitshift
        return None;
    }
    let subset_count = 1u64 << elements.len(); //TODO: dedupe
    Some(
        (0..subset_count)
            .map(|mask| Subset::new(mask, elements))
            .collect(),
    )
}

fn naive_subsets_in_range<T: Arith>(
    elements: &[T],
    range: Range<T>,
) -> Option<(Vec<Subset<T, u64>>)> {
    let mut subsets = all_subsets(elements)?;
    subsets.retain(|subset| range.contains(&subset.sum));
    Some(subsets)
}

struct Submasks {
    mask: u64,
    submask: u64,
    start: bool,
}
impl Iterator for Submasks where {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        if self.start {
            self.start = false;
            return Some(self.mask);
        }
        if self.submask == 0 {
            return None;
        }
        self.submask -= 1;
        self.submask &= self.mask;
        Some(self.submask)
    }
}

fn submasks(mask: u64) -> Submasks {
    Submasks {
        mask,
        submask: mask,
        start: true,
    }
}

fn split_mask<T: Arith>(mask: u64, elements: &[T]) -> (u64, u64) {
    let mut element_masks = Vec::with_capacity(mask.count_ones() as usize);
    for i in 0..64 {
        if (mask & 1 << i) > 0 {
            element_masks.push((elements[i], 1 << i));
        }
    }
    element_masks.sort_unstable();
    let (smalls, larges) = element_masks.split_at(element_masks.len() / 2);
    let (mut small_mask, mut large_mask) = (0, 0);
    for (_, element_mask) in smalls {
        small_mask |= element_mask;
    }
    for (_, element_mask) in larges {
        large_mask |= element_mask;
    }
    (small_mask, large_mask)
}

struct EHS<T> {
    ascending: Vec<Subset<T, u64>>,
    descending: Vec<Subset<T, u64>>,
    ascending_cutoff: usize,
    ascending_index: usize,
    range: Range<T>,
}
impl<'a, T: Arith> Iterator for EHS<T> where {
    type Item = Subset<T, u64>;
    fn next(&mut self) -> Option<Subset<T, u64>> {
        let out;
        {
            let descending = self.descending.last()?;
            let ascending = &self.ascending[self.ascending_index];
            assert_eq!(0, ascending.mask & descending.mask);
            out = Subset {
                sum: ascending.sum + descending.sum,
                mask: ascending.mask | descending.mask,
            };
        }
        match self.ascending_index.checked_add(1) {
            None => {
                self.ascending_index = 0;
                self.step_descending();
            }
            Some(ascending_index) => {
                if ascending_index >= self.ascending_cutoff {
                    self.step_descending();
                }
            }
        }
        Some(out)
    }
}
impl<T: Arith> EHS<T> where {
    fn new(mask: u64, elements: &[T], range: Range<T>) -> Self {
        let (left, right) = split_mask(mask, elements);
        let ascending: Vec<Subset<T, u64>> = submasks(left)
            .map(|mask| Subset::new(mask, elements))
            .collect();
        let descending = submasks(right)
            .map(|mask| Subset::new(mask, elements))
            .collect();
        let ascending_cutoff = ascending.len();
        //Note: this is invalid: we're relying on set_ascending to clean it up.
        let ascending_index = ascending.len();
        let mut ehs = EHS {
            ascending,
            descending,
            ascending_cutoff,
            ascending_index,
            range,
        };
        ehs.set_ascending();
        ehs
    }
    fn step_descending(&mut self) {
        self.descending
            .pop()
            .expect("Called step_descending with empty descending");
        if self.descending.len() > 0 {
            self.set_ascending();
        }
    }
    fn set_ascending(&mut self) {
        let last_descending = self
            .descending
            .last()
            .expect("Called set_ascending with empty descending");
        while (self.ascending.len() > 0)
            && (self.ascending.last().unwrap().sum + last_descending.sum < self.range.start)
        {
            self.ascending.pop();
        }
        self.ascending_cutoff = min(self.ascending_cutoff, self.ascending.len());
        while (self.ascending_cutoff > 0)
            && (self.ascending[self.ascending_cutoff - 1].sum + last_descending.sum
                < self.range.end)
        {
            self.ascending_cutoff -= 1;
        }
        self.ascending_index = self.ascending_cutoff;
    }
}

#[cfg(test)]
mod tests {
    use ehs::{all_subsets, naive_subsets_in_range, submasks, Subset, EHS};
    use proptest::collection::vec;
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::hash::Hash;
    #[test]
    fn unit_all_subsets() {
        let elements = vec![1, 2, 3];
        let subsets = all_subsets(&elements).unwrap();
        let expected = vec![
            Subset { mask: 0, sum: 0 },
            Subset { mask: 1, sum: 1 },
            Subset { mask: 2, sum: 2 },
            Subset { mask: 3, sum: 3 },
            Subset { mask: 4, sum: 3 },
            Subset { mask: 5, sum: 4 },
            Subset { mask: 6, sum: 5 },
            Subset { mask: 7, sum: 6 },
        ];
        assert_eq!(subsets, expected);
    }
    #[test]
    fn unit_submasks() {
        let mask = 0b10101u64;
        let mut expected = vec![
            0b00000u64, 0b00001u64, 0b00100u64, 0b00101u64, 0b10000u64, 0b10001u64, 0b10100u64,
            0b10101u64,
        ];
        expected.reverse();
        let submasks_iterator = submasks(mask);
        let actual: Vec<u64> = submasks_iterator.collect();
        assert_eq!(actual, expected);
    }
    fn assert_permutation<T: Hash + Eq + Debug, I1: Iterator<Item = T>, I2: Iterator<Item = T>>(
        left: I1,
        right: I2,
    ) {
        let mut counts = HashMap::new();
        for item in left {
            let (left_count, _) = counts.entry(item).or_insert((0, 0));
            *left_count += 1;
        }
        for item in right {
            let (_, right_count) = counts.entry(item).or_insert((0, 0));
            *right_count += 1;
        }
        counts.retain(|_, (l, r)| l != r);
        if counts.len() == 0 {
            return;
        }
        panic!(
            "Left and right had the following mismatched counts: {:?}",
            counts
        )
    }
    proptest! {
        #[test]
        fn prop_ehs(ref elements in vec(1i32..100, 1..10), b1 in 1i32..100, b2 in 1i32..100) {
            let range = if b1 > b2 {
                b1..b2
            } else {
                b2..b1
            };
            let mask = (1 << elements.len() -1);
            assert_permutation(naive_subsets_in_range(elements, range.clone()).unwrap().into_iter(), EHS::new(mask, elements, range))
       }
    }
}
