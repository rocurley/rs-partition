use super::arith::Arith;
use super::subset::{ordered_subsets, split_mask, Down, OrderedSubsets, Subset, Up};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::Peekable;
use std::ops::Range;

#[derive(Debug)]
pub struct ESS<T, I1: Iterator<Item = Subset<T, u64>>, I2: Iterator<Item = Subset<T, u64>>> {
    ascending: LazyQueue<Subset<T, u64>, I1>,
    ascending_index: usize,
    descending: Peekable<I2>,
    pub range: Range<T>, //TODO: make this private, add a "replace range" method
}

#[derive(Debug)]
struct LazyQueue<T, I>
where
    I: Iterator<Item = T>,
{
    cached: VecDeque<T>,
    rest: I,
}
impl<T, I: Iterator<Item = T>> LazyQueue<T, I> {
    fn cache_through(&mut self, index: usize) -> Option<()> {
        while self.cached.len() <= index {
            self.cached.push_back(self.rest.next()?);
        }
        Some(())
    }
    fn pop(&mut self) -> Option<T> {
        self.cache_through(0);
        self.cached.pop_front()
    }
    fn get(&mut self, index: usize) -> Option<&T> {
        self.cache_through(index)?;
        Some(&self.cached[index])
    }
    fn new(iter: I) -> Self {
        Self {
            cached: VecDeque::new(),
            rest: iter,
        }
    }
}
impl<T: Arith, I1: Iterator<Item = Subset<T, u64>>, I2: Iterator<Item = Subset<T, u64>>> Iterator
    for ESS<T, I1, I2> where
{
    type Item = Subset<T, u64>;
    fn next(&mut self) -> Option<Subset<T, u64>> {
        let descending = self.descending.peek()?;
        let ascending = match self.ascending.get(self.ascending_index) {
            Some(ascending) => ascending,
            None => {
                self.step_descending();
                return self.next();
            }
        };
        assert_eq!(0, ascending.mask & descending.mask);
        let out = Subset::union(ascending, descending);
        match (out.sum < self.range.start, self.range.end <= out.sum) {
            (false, false) => {
                //In range
                self.ascending_index += 1;
                Some(out)
            }
            (false, true) => {
                //Too big. Ascending will only increase, so drop the current descending.
                self.step_descending();
                self.next()
            }
            (true, false) => {
                //Too small. Descending will only decrease, so drop the current ascending.
                self.ascending.pop();
                self.next()
            }
            (true, true) => unreachable!(),
        }
    }
}
pub fn iterate_subsets_in_range<T: Arith>(
    mask: u64,
    elements: &[T],
    range: Range<T>,
) -> ESS<
    T,
    impl Iterator<Item = Subset<T, u64>> + Debug,
    impl Iterator<Item = Subset<T, u64>> + Debug,
> {
    let (left, right) = split_mask(mask, elements);
    let ascending_raw: OrderedSubsets<_, Up> = ordered_subsets(left, elements);
    let ascending = LazyQueue::new(ascending_raw);
    let descending_raw: OrderedSubsets<_, Down> = ordered_subsets(right, elements);
    let descending = descending_raw.peekable();
    let ascending_index = 0;
    ESS {
        ascending,
        descending,
        ascending_index,
        range,
    }
}

impl<T: Arith, I1: Iterator<Item = Subset<T, u64>>, I2: Iterator<Item = Subset<T, u64>>>
    ESS<T, I1, I2> where
{
    fn step_descending(&mut self) {
        self.descending
            .next()
            .expect("Called step_descending with empty descending");
        self.ascending_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use arith::Arith;
    use ess::{iterate_subsets_in_range, Subset};
    use proptest::collection::vec;
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::ops::Range;
    use subset::{all_subsets, submasks};

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
        let mask = 0b_10101_u64;
        let mut expected = vec![
            0b_00000, 0b_00001, 0b_00100, 0b_00101, 0b_10000, 0b_10001, 0b_10100,
            0b_10101,
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
        if counts.is_empty() {
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
        assert_permutation(left.iter(), right.iter());
    }
    #[test]
    #[should_panic]
    fn unit_not_permutation_2() {
        let left = [1, 2, 3];
        let right = [1, 2, 3, 4];
        assert_permutation(left.iter(), right.iter());
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
    fn test_iterate_subsets_in_range(elements: &[i32], range: Range<i32>) {
        let mask = (1 << elements.len()) - 1;
        let expected: Vec<Subset<i32, u64>> =
            naive_subsets_in_range(elements, range.clone()).unwrap();
        let actual = iterate_subsets_in_range(mask, elements, range);
        assert_permutation(expected.into_iter(), actual);
    }

    proptest! {
        #[test]
        fn prop_iterate_subsets_in_range(ref elements in vec(1_i32..100, 1..10), b1 in 1_i32..100, b2 in 1_i32..100) {
            let range = if b1 < b2 {
                b1..b2
            } else {
                b2..b1
            };
            test_iterate_subsets_in_range(elements, range);
       }
    }
    #[test]
    fn unit_iterate_subsets_in_range_1() {
        let elements = [2, 1];
        let range = 1..4;
        test_iterate_subsets_in_range(&elements, range);
    }
    #[test]
    fn unit_iterate_subsets_in_range_2() {
        let elements = [24, 17, 24, 25, 25];
        let range = 58..66;
        let mask = (1 << elements.len()) - 1;
        let expected = vec![Subset::new(0b00111, &elements)];
        let actual = iterate_subsets_in_range(mask, &elements, range);
        assert_permutation(expected.into_iter(), actual);
    }
}
