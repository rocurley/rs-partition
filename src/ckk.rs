use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::iter::Iterator;
use std::mem::swap;

#[path = "arith.rs"]
pub mod arith;
use self::arith::Arith;

#[derive(Eq, Debug, Clone)]
pub struct KKPartition<T: Arith> {
    left: Vec<T>,
    right: Vec<T>,
    score: T,
}

impl<T: Arith> PartialEq for KKPartition<T> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
impl<T: Arith> PartialOrd for KKPartition<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.score.cmp(&other.score))
    }
}
impl<T: Arith> Ord for KKPartition<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl<T: Arith> KKPartition<T> {
    fn merge(&mut self, mut other: Self) {
        self.left.append(&mut other.right);
        self.right.append(&mut other.left);
        self.score -= other.score;
    }
    fn merge_rev(&mut self, mut other: Self) {
        self.left.append(&mut other.left);
        self.right.append(&mut other.right);
        self.score += other.score;
    }

    fn singleton(x: T) -> Self {
        KKPartition {
            left: vec![x],
            right: Vec::new(),
            score: x,
        }
    }
}

#[derive(Debug)]
pub enum RNPResult<T: Arith> {
    TwoWay(KKPartition<T>),
    EvenSplit(Box<RNPResult<T>>, Box<RNPResult<T>>),
    OddSplit(Vec<T>, Box<RNPResult<T>>),
}
impl<T: Arith> RNPResult<T> {
    pub fn to_vec(&self) -> Vec<&[T]> {
        match self {
            RNPResult::TwoWay(kk) => vec![&kk.left, &kk.right],
            RNPResult::EvenSplit(l, r) => {
                let mut v = l.to_vec();
                v.append(&mut r.to_vec());
                v
            }
            RNPResult::OddSplit(first, rest) => {
                let mut v = vec![first.as_slice()];
                v.append(&mut rest.to_vec());
                v
            }
        }
    }
}

pub fn kk<T: Arith>(elements: &[T]) -> KKPartition<T> {
    let mut heap: BinaryHeap<KKPartition<T>> = elements
        .into_iter()
        .map(|&x| KKPartition::singleton(x))
        .collect();
    loop {
        match (heap.pop(), heap.pop()) {
            (None, None) => panic!("heap is empty"),
            (None, Some(_)) => panic!("first empty, snd not"),
            (Some(first), None) => return first,
            (Some(mut first), Some(snd)) => {
                first.merge(snd);
                heap.push(first);
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
    let mut heap: BinaryHeap<KKPartition<T>> = elements
        .into_iter()
        .map(|&x| KKPartition::singleton(x))
        .collect();
    for direction in directions {
        let mut first = heap.pop().expect("heap is empty");
        match heap.pop() {
            None => return first,
            Some(mut snd) => {
                match direction {
                    Direction::Diff => first.merge(snd),
                    Direction::Sum => first.merge_rev(snd),
                }
                heap.push(first);
            }
        }
    }
    let mut first = heap.pop().expect("heap is empty");
    for p in heap {
        first.merge(p);
    }
    return first;
}

pub fn ckk_old<T: Arith>(elements: &[T]) -> KKPartition<T> {
    let mut best_directions = Vec::with_capacity(elements.len());
    let mut directions = Vec::with_capacity(elements.len());
    let heap = elements.iter().map(|x| x.clone()).collect();
    let mut best = elements.iter().map(|x| x.clone()).sum();
    ckk_raw_old(heap, &mut directions, &mut best, &mut best_directions);
    reconstruct_ckk(elements, best_directions)
}

fn ckk_raw_old<T: Arith>(
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
            let sum_rest: T = heap.iter().map(|x| *x).sum();
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
            ckk_raw_old(new_heap, directions, best, best_directions);
            directions.pop();
            directions.push(Direction::Sum);
            heap.push(first + snd);
            ckk_raw_old(heap, directions, best, best_directions);
            directions.pop();
        }
    }
}

pub fn ckk<T: Arith>(elements: &[T]) -> KKPartition<T> {
    let mut best_directions = Vec::with_capacity(elements.len());
    let mut directions = Vec::with_capacity(elements.len());
    let mut best = elements.iter().map(|x| x.clone()).sum();
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
    let original_first = first.clone();
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

pub fn rnp<T: Arith>(elements: &[T]) -> RNPResult<T> {
    let mut upper_bound = n_kk_score(elements, 4);
    let mut best = None;
    let heap: BinaryHeap<KKPartition<T>> = elements
        .into_iter()
        .map(|&x| KKPartition::singleton(x))
        .collect();
    rnp_helper(heap, &mut upper_bound, &mut best);
    best.expect("KK heursitic was optimal, which isn't properly handled yet :(")
}

fn rnp_helper<T: Arith>(
    mut heap: BinaryHeap<KKPartition<T>>,
    upper_bound: &mut T,
    best: &mut Option<RNPResult<T>>,
) {
    let mut first = heap.pop().expect("heap is empty");
    match heap.pop() {
        None => {
            if first.score / 2.into() < *upper_bound {
                let left = ckk(&first.left);
                if (left.score + first.score) / 2.into() < *upper_bound {
                    let right = ckk(&first.right);
                    let score = (first.score + right.score + left.score) / 2.into();
                    if score < *upper_bound {
                        *upper_bound = score;
                        *best = Some(RNPResult::EvenSplit(
                            Box::new(RNPResult::TwoWay(left)),
                            Box::new(RNPResult::TwoWay(right)),
                        ));
                    }
                }
            }
        }
        Some(snd) => {
            let rest_score = snd.score + heap.iter().map(|p| p.score).sum();
            if first.score > rest_score {
                if (first.score - rest_score) / 2.into() > *upper_bound {
                    return;
                }
            }
            let mut new_heap = heap.clone();
            let mut new_first = first.clone();
            let new_snd = snd.clone();
            new_first.merge(new_snd);
            new_heap.push(new_first);
            rnp_helper(new_heap, upper_bound, best);
            first.merge_rev(snd);
            heap.push(first);
            rnp_helper(heap, upper_bound, best);
        }
    }
}
#[derive(Eq, Debug, Clone)]
struct NKKPartition<T: Arith> {
    elements: Vec<Vec<T>>,
}

impl<T: Arith> PartialEq for NKKPartition<T> {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}
impl<T: Arith> PartialOrd for NKKPartition<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.score().cmp(&other.score()))
    }
}
impl<T: Arith> Ord for NKKPartition<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

impl<T: Arith> NKKPartition<T> {
    fn score(&self) -> T {
        let max: T = self.elements[0].iter().map(|x| *x).sum();
        let min: T = self.elements[self.elements.len() - 1]
            .iter()
            .map(|x| *x)
            .sum();
        max - min
    }
    fn merge(&mut self, other: Self) {
        for (s, mut o) in self
            .elements
            .iter_mut()
            .zip(other.elements.into_iter().rev())
        {
            s.append(&mut o);
        }
        self.elements.sort_unstable_by(|x, y| {
            let y_sum: T = y.iter().map(|&v| v).sum();
            let x_sum: T = x.iter().map(|&v| v).sum();
            y_sum.cmp(&x_sum)
        });
    }
    fn singleton(x: T, n: usize) -> Self {
        let mut elements = vec![Vec::new(); n];
        elements[0].push(x);
        NKKPartition { elements: elements }
    }
}

pub fn n_kk_score<T: Arith>(elements: &[T], n: usize) -> T {
    let mut heap: BinaryHeap<NKKPartition<T>> = elements
        .into_iter()
        .map(|&x| NKKPartition::singleton(x, n))
        .collect();
    loop {
        let mut first = heap.pop().expect("heap is empty");
        match heap.pop() {
            None => return first.score(),
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
    use ckk::{ckk, ckk_old, ckk_raw, ckk_raw_old, rnp};
    use gcc::find_best_partitioning;
    use proptest::collection::vec;
    use std::borrow::{Borrow, BorrowMut};
    proptest! {
        #[test]
        fn prop_compare_raw(ref elements in vec(1i32..100, 1..10)) {
            let mut best_directions_1 = Vec::with_capacity(elements.len());
            let mut directions_1 = Vec::with_capacity(elements.len());
            let heap = elements.iter().map(|x| x.clone()).collect();
            let mut best_1 = elements.iter().map(|x| x.clone()).sum();
            ckk_raw_old(heap, &mut directions_1, &mut best_1, &mut best_directions_1);
            let mut best_directions_2 = Vec::with_capacity(elements.len());
            let mut directions_2 = Vec::with_capacity(elements.len());
            let mut best_2 = elements.iter().map(|x| x.clone()).sum();
            let mut work_elements_2 = elements.to_vec();
            let sum = elements.iter().cloned().sum();
            ckk_raw(&mut work_elements_2, sum, &mut directions_2, &mut best_2, &mut best_directions_2);
            assert_eq!(best_directions_1,  best_directions_2);
       }
    }
    proptest! {
        #[test]
        fn prop_compare_ckk(ref elements in vec(1i32..100, 2..10)) {
            let partition_1 = ckk_old(elements);
            let partition_2 = ckk(elements);
            assert_eq!(partition_1, partition_2);
       }
    }
    #[test]
    fn unit_ckk() {
        let elements = vec![2, 3, 4, 5];
        let partition = ckk(&elements);
        assert_eq!(partition.score, 0);
    }
    proptest! {
        #[test]
        fn prop_rnp_gcc(ref elements in vec(1i32..100, 1..10)) {
            let (gcc_results, _) = find_best_partitioning(4, &elements);
            let gcc_sums : Vec<i32> = gcc_results.to_vec().into_iter().map(|p| p.sum()).collect();
            let gcc_score = gcc_sums.iter().max().unwrap() - gcc_sums.iter().min().unwrap();
            let rnp_results = rnp(&elements);
            let rnp_sums : Vec<i32> = rnp_results.to_vec().into_iter().map(|p| p.iter().sum()).collect();
            let rnp_score = rnp_sums.iter().max().unwrap() - gcc_sums.iter().min().unwrap();
            /*
            let mut gcc_sorted : Vec<Vec<i32>> = gcc_results.iter_mut().map(|p| {
                let mut els = p.to_vec();
                els.sort();
                els
            }).collect();
            gcc_sorted.sort();
            let mut rnp_sorted : Vec<Vec<i32>> = rnp_results.to_vec().into_iter().map(|els| {
                let mut vec = els.to_vec();
                vec.sort();
                vec
            }).collect();
            rnp_sorted.sort();
            */
            assert_eq!(rnp_score, gcc_score);
       }
    }
    #[bench]
    fn bench_ckk(b: &mut Bencher) {
        let elements = vec![
            403188, 4114168, 4114168, 5759835, 5759835, 5759835, 2879917, 8228336, 8228336,
            8228336, 8228336, 8228336, 8228336, 8228336, 2057084, 2057084, 2057084, 2057084,
            2057084, 2057084, 2057084, 9599726, 9599726, 9599726, 9599726, 9599726, 9599726,
            537584, 537584, 537584,
        ];
        b.iter(|| ckk_old(&elements));
    }
    #[bench]
    fn bench_ckk_2(b: &mut Bencher) {
        let elements = vec![
            403188, 4114168, 4114168, 5759835, 5759835, 5759835, 2879917, 8228336, 8228336,
            8228336, 8228336, 8228336, 8228336, 8228336, 2057084, 2057084, 2057084, 2057084,
            2057084, 2057084, 2057084, 9599726, 9599726, 9599726, 9599726, 9599726, 9599726,
            537584, 537584, 537584,
        ];
        b.iter(|| ckk(&elements));
    }
    #[bench]
    fn bench_rnp(b: &mut Bencher) {
        let elements = vec![
            403188, 4114168, 4114168, 5759835, 5759835, 5759835, 2879917, 8228336, 8228336,
            8228336, 8228336, 8228336, 8228336, 8228336, 2057084, 2057084, 2057084, 2057084,
            537584, 537584, 537584,
        ];
        b.iter(|| rnp(&elements));
    }
}
