use super::arith::Arith;
use super::subset::Subset;
use itertools::Itertools;
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::iter::Iterator;
use std::mem::swap;

#[derive(Eq, Debug, Clone)]
pub struct KKPartition<T: Arith> {
    pub left: Subset<T, u64>,
    pub right: Subset<T, u64>,
}

impl<T: Arith> PartialEq for KKPartition<T> {
    fn eq(&self, other: &Self) -> bool {
        self.delta() == other.delta()
    }
}
impl<T: Arith> PartialOrd for KKPartition<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.delta().cmp(&other.delta()))
    }
}
impl<T: Arith> Ord for KKPartition<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.delta().cmp(&other.delta())
    }
}

impl<T: Arith> KKPartition<T> {
    pub fn delta(&self) -> T {
        self.left.sum - self.right.sum
    }
    pub fn merge(big: &Self, small: &Self) -> Self {
        let left = Subset::union(&big.left, &small.right);
        let right = Subset::union(&big.right, &small.left);
        KKPartition { left, right }
    }
    pub fn merge_rev(big: &Self, small: &Self) -> Self {
        let left = Subset::union(&big.left, &small.left);
        let right = Subset::union(&big.right, &small.right);
        KKPartition { left, right }
    }

    pub fn singleton(index: usize, elements: &[T]) -> Self {
        Self {
            left: Subset::from_index(index, elements),
            right: Subset::empty(),
        }
    }
    pub fn score(&self) -> T {
        self.left.sum
    }
}

pub fn kk<T: Arith>(elements: &[T]) -> KKPartition<T> {
    let mut heap: BinaryHeap<KKPartition<T>> = (0..elements.len())
        .map(|i| KKPartition::singleton(i, elements))
        .collect();
    loop {
        match (heap.pop(), heap.pop()) {
            (None, None) => panic!("heap is empty"),
            (None, Some(_)) => panic!("first empty, snd not"),
            (Some(first), None) => return first,
            (Some(first), Some(snd)) => {
                heap.push(KKPartition::merge(&first, &snd));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Diff,
    Sum,
}

fn reconstruct_ckk<T: Arith>(elements: &[T], directions: Vec<Direction>) -> KKPartition<T> {
    let mut heap: BinaryHeap<KKPartition<T>> = (0..elements.len())
        .map(|i| KKPartition::singleton(i, elements))
        .collect();
    for direction in directions {
        let first = heap.pop().expect("heap is empty");
        match heap.peek_mut() {
            None => return first,
            Some(mut snd) => {
                *snd = match direction {
                    Direction::Diff => KKPartition::merge(&first, &snd),
                    Direction::Sum => KKPartition::merge_rev(&first, &snd),
                };
            }
        }
    }
    heap.into_iter()
        .fold1(|acc, next| KKPartition::merge(&acc, &next))
        .expect("heap is empty")
}

pub fn old<T: Arith>(elements: &[T]) -> KKPartition<T> {
    let mut best_directions = Vec::with_capacity(elements.len());
    let mut directions = Vec::with_capacity(elements.len());
    let heap = elements.iter().cloned().collect();
    let mut best = elements.iter().cloned().sum();
    old_raw(heap, &mut directions, &mut best, &mut best_directions);
    reconstruct_ckk(elements, best_directions)
}

fn old_raw<T: Arith>(
    mut heap: BinaryHeap<T>,
    directions: &mut Vec<Direction>,
    best: &mut T,
    best_directions: &mut Vec<Direction>,
) {
    let first = heap.pop().expect("heap is empty");
    match heap.pop() {
        None => {
            if *best > first {
                *best = first;
                best_directions.clone_from(directions);
            }
        }
        Some(snd) => {
            let sum_rest: T = heap.iter().cloned().sum();
            if first >= snd + sum_rest {
                let best_possible_score = first - snd - sum_rest;
                if *best > best_possible_score {
                    *best = best_possible_score;
                    best_directions.clone_from(directions);
                }
                return;
            }
            let mut new_heap = heap.clone();
            new_heap.push(first - snd);
            directions.push(Direction::Diff);
            old_raw(new_heap, directions, best, best_directions);
            directions.pop();
            directions.push(Direction::Sum);
            heap.push(first + snd);
            old_raw(heap, directions, best, best_directions);
            directions.pop();
        }
    }
}

pub fn ckk<T: Arith>(elements: &[T]) -> KKPartition<T> {
    let mut best_directions = Vec::with_capacity(elements.len());
    let mut directions = Vec::with_capacity(elements.len());
    let mut best = elements.iter().cloned().sum();
    let mut work_elements = elements.to_vec();
    let sum = elements.iter().cloned().sum();
    ckk_raw(
        &mut work_elements,
        sum,
        &mut directions,
        &mut best,
        &mut best_directions,
    );
    reconstruct_ckk(elements, best_directions)
}

pub fn from_subset<T: Arith>(subset: &Subset<T, u64>, elements: &[T]) -> KKPartition<T> {
    let mut best_directions = Vec::with_capacity(elements.len());
    let mut directions = Vec::with_capacity(elements.len());
    let mut best = elements.iter().cloned().sum();
    let mut work_elements: Vec<T> = subset.elements(elements).collect();
    let sum = subset.sum;
    ckk_raw(
        &mut work_elements,
        sum,
        &mut directions,
        &mut best,
        &mut best_directions,
    );
    reconstruct_ckk(elements, best_directions)
}

// When ckk_raw returns, elements must:
// * Have an unchanged first element.
// * Otherwise be a permutation of its original value.
fn ckk_raw<T: Arith>(
    elements: &mut [T],
    sum: T,
    directions: &mut Vec<Direction>,
    best: &mut T,
    best_directions: &mut Vec<Direction>,
) {
    let (first, tail) = elements.split_first_mut().expect("elements is empty");
    let original_first = *first;
    let snd_val: T = match tail.split_first_mut() {
        None => {
            if *best > *first {
                *best = *first;
                best_directions.clone_from(directions);
            }
            return;
        }
        Some((snd, rest)) => {
            //Pull the two largest values to the front
            if *snd > *first {
                swap(first, snd);
            }
            for x in rest.iter_mut() {
                if *x > *snd {
                    swap(x, snd);
                    if *snd > *first {
                        swap(first, snd);
                    }
                }
            }
            *snd
        }
    };
    let sum_rest = sum - *first;
    if *first >= sum_rest {
        let best_possible_score = *first - sum_rest;
        if *best > best_possible_score {
            *best = best_possible_score;
            best_directions.clone_from(directions);
        }
        if *first == original_first {
            return;
        }
        for x in tail {
            if *x == original_first {
                swap(first, x);
                return;
            }
        }
        panic!("Couldn't find the original first");
    }
    directions.push(Direction::Diff);
    tail[0] = *first - snd_val;
    ckk_raw(
        tail,
        sum - snd_val - snd_val,
        directions,
        best,
        best_directions,
    );
    directions.pop();
    directions.push(Direction::Sum);
    tail[0] = *first + snd_val;
    ckk_raw(tail, sum, directions, best, best_directions);
    directions.pop();
    tail[0] = snd_val;
    if *first == original_first {
        return;
    }
    for x in tail {
        if *x == original_first {
            swap(first, x);
            return;
        }
    }
    panic!("Couldn't find the original first");
}

#[derive(Eq, Debug, Clone)]
pub struct Partitioning<T: Arith> {
    pub partitions: Vec<Subset<T, u64>>,
}

impl<T: Arith> PartialEq for Partitioning<T> {
    fn eq(&self, other: &Self) -> bool {
        self.delta() == other.delta()
    }
}
impl<T: Arith> PartialOrd for Partitioning<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.delta().cmp(&other.delta()))
    }
}
impl<T: Arith> Ord for Partitioning<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.delta().cmp(&other.delta())
    }
}

impl<T: Arith> Partitioning<T> {
    pub fn score(&self) -> T {
        self.partitions[0].sum
    }
    pub fn delta(&self) -> T {
        let max = self.partitions[0].sum;
        let min = self.partitions.last().unwrap().sum;
        max - min
    }
    fn merge(&mut self, other: Self) {
        for (s, o) in self
            .partitions
            .iter_mut()
            .zip(other.partitions.into_iter().rev())
        {
            *s = Subset::union(&s, &o);
        }
        self.partitions.sort_unstable_by_key(|x| Reverse(x.sum));
    }
    fn singleton(mask: u64, elements: &[T], n: u8) -> Self {
        let mut partitions = vec![Subset::empty(); n as usize];
        partitions[0] = Subset::new(mask, elements);
        Self { partitions }
    }
}

pub fn n_kk<T: Arith>(elements: &[T], n: u8) -> Partitioning<T> {
    let mut heap: BinaryHeap<Partitioning<T>> = (0..elements.len())
        .map(|i| Partitioning::singleton(1 << i, elements, n))
        .collect();
    loop {
        let mut first = heap.pop().expect("heap is empty");
        match heap.pop() {
            None => return first,
            Some(snd) => {
                first.merge(snd);
                heap.push(first);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use ckk::{ckk, ckk_raw, kk, n_kk, old, old_raw};
    use proptest::collection::vec;
    proptest! {
        #[test]
        fn prop_compare_raw(ref elements in vec(1_i32..100, 1..10)) {
            let mut best_directions_1 = Vec::with_capacity(elements.len());
            let mut directions_1 = Vec::with_capacity(elements.len());
            let heap = elements.iter().cloned().collect();
            let mut best_1 = elements.iter().cloned().sum();
            old_raw(heap, &mut directions_1, &mut best_1, &mut best_directions_1);
            let mut best_directions_2 = Vec::with_capacity(elements.len());
            let mut directions_2 = Vec::with_capacity(elements.len());
            let mut best_2 = elements.iter().cloned().sum();
            let mut work_elements_2 = elements.to_vec();
            let sum = elements.iter().cloned().sum();
            ckk_raw(&mut work_elements_2, sum, &mut directions_2, &mut best_2, &mut best_directions_2);
            assert_eq!(best_directions_1,  best_directions_2);
       }
    }
    proptest! {
        #[test]
        fn prop_compare_ckk(ref elements in vec(1_i32..100, 2..10)) {
            let partition_1 = old(elements);
            let partition_2 = ckk(elements);
            assert_eq!(partition_1, partition_2);
       }
    }
    #[test]
    fn unit_ckk() {
        let elements = vec![2, 3, 4, 5];
        let partition = ckk(&elements);
        assert_eq!(partition.score(), 7, "partiton was {:?}", partition);
    }
    proptest! {
        #[test]
        fn prop_n_kk(ref elements in vec(1_i32..100, 1..10)) {
            let partition_1 = kk(elements).score();
            let partition_2 = n_kk(elements,2).score();
            assert_eq!(partition_1, partition_2);
       }
    }
    #[bench]
    fn bench_ckk(b: &mut Bencher) {
        #[allow(clippy::unreadable_literal)]
        let elements = vec![
            403188, 4114168, 4114168, 5759835, 5759835, 5759835, 2879917, 8228336, 8228336,
            8228336, 8228336, 8228336, 8228336, 8228336, 2057084, 2057084, 2057084, 2057084,
            2057084, 2057084, 2057084, 9599726, 9599726, 9599726, 9599726, 9599726, 9599726,
            537584, 537584, 537584,
        ];
        b.iter(|| old(&elements));
    }
    #[bench]
    fn bench_ckk_2(b: &mut Bencher) {
        #[allow(clippy::unreadable_literal)]
        let elements = vec![
            403188, 4114168, 4114168, 5759835, 5759835, 5759835, 2879917, 8228336, 8228336,
            8228336, 8228336, 8228336, 8228336, 8228336, 2057084, 2057084, 2057084, 2057084,
            2057084, 2057084, 2057084, 9599726, 9599726, 9599726, 9599726, 9599726, 9599726,
            537584, 537584, 537584,
        ];
        b.iter(|| ckk(&elements));
    }
}
