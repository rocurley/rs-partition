use super::arith::Arith;
use super::subset::{split_mask, submasks, Subset};
use std::cmp::Reverse;
use std::ops::Range;

#[derive(Debug)]
struct EHS<T> {
    ascending: Vec<Subset<T, u64>>,
    descending: Vec<Subset<T, u64>>,
    ascending_index: usize,
    range: Range<T>,
}
impl<'a, T: Arith> Iterator for EHS<T> where {
    type Item = Subset<T, u64>;
    fn next(&mut self) -> Option<Subset<T, u64>> {
        if self.descending.len() == 0 {
            return None;
        }
        if self.ascending_index == 0 {
            self.step_descending();
            return self.next();
        }
        self.ascending_index -= 1;
        let out;
        {
            let descending = self
                .descending
                .last()
                .expect("Empty descending after check");
            let ascending = &self.ascending[self.ascending_index];
            assert_eq!(0, ascending.mask & descending.mask);
            out = Subset {
                sum: ascending.sum + descending.sum,
                mask: ascending.mask | descending.mask,
            };
        }
        if self.range.contains(&out.sum) {
            return Some(out);
        }
        self.step_descending();
        self.next()
    }
}
impl<T: Arith> EHS<T> where {
    pub fn new(mask: u64, elements: &[T], range: Range<T>) -> Self {
        let (left, right) = split_mask(mask, elements);
        let mut ascending: Vec<Subset<T, u64>> = submasks(left)
            .map(|mask| Subset::new(mask, elements))
            .collect();
        ascending.sort_by_key(|subset| Reverse(subset.sum));
        let mut descending: Vec<Subset<T, u64>> = submasks(right)
            .map(|mask| Subset::new(mask, elements))
            .collect();
        descending.sort_by_key(|subset| subset.sum);
        let ascending_index = ascending.len();
        let mut ehs = EHS {
            ascending,
            descending,
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
        if self.descending.len() == 0 {
            return;
        }
        self.set_ascending();
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
        self.ascending_index = self.ascending.len();
    }
}

#[cfg(test)]
mod tests {
    use arith::Arith;
    use ehs::{submasks, Subset, EHS};
    use proptest::collection::vec;
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::ops::Range;
    use subset::all_subsets;

    fn naive_subsets_in_range<T: Arith>(
        elements: &[T],
        range: Range<T>,
    ) -> Option<(Vec<Subset<T, u64>>)> {
        let mut subsets = all_subsets(elements)?;
        subsets.retain(|subset| range.contains(&subset.sum));
        Some(subsets)
    }

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
    #[test]
    #[should_panic]
    fn unit_not_permutation_1() {
        let left = [];
        let right = [1, 2, 3, 4];
        assert_permutation(left.into_iter(), right.into_iter());
    }
    #[test]
    #[should_panic]
    fn unit_not_permutation_2() {
        let left = [1, 2, 3];
        let right = [1, 2, 3, 4];
        assert_permutation(left.into_iter(), right.into_iter());
    }
    #[test]
    fn unit_naive_subsets() {
        let elements = [1, 3, 5];
        let range = 3..6;
        let expected = vec![
            Subset::new(0b010, &elements),
            Subset::new(0b011, &elements),
            Subset::new(0b100, &elements),
        ];
        let actual = naive_subsets_in_range(&elements, range).unwrap();
        assert_eq!(&expected, &actual);
    }
    proptest! {
        #[test]
        fn prop_ehs(ref elements in vec(1i32..100, 1..10), b1 in 1i32..100, b2 in 1i32..100) {
            let range = if b1 < b2 {
                b1..b2
            } else {
                b2..b1
            };
            let mask = (1 << elements.len()) -1;
            let expected : Vec<Subset<i32, u64>> =
                naive_subsets_in_range(elements, range.clone()).unwrap();
            let actual = EHS::new(mask, elements, range);
            assert_permutation(
                expected.into_iter(),
                actual
            );
       }
    }
}
