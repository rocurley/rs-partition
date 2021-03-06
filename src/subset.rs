use super::arith::Arith;
use std::cmp::Ordering;
use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::swap;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Subset<T, M> {
    pub sum: T,
    pub mask: M,
}
impl<T: Arith> Subset<T, u64> {
    pub fn new(mask: u64, elements: &[T]) -> Self {
        let mut selected_bit = 1;
        let mut sum = T::from(0);
        for x in elements {
            if mask & selected_bit > 0 {
                sum += *x;
            }
            selected_bit <<= 1;
        }
        Self { sum, mask }
    }
    pub fn from_index(i: usize, elements: &[T]) -> Self {
        let sum = elements[i];
        let mask = 1 << i;
        Self { sum, mask }
    }
    pub fn empty() -> Self {
        Self {
            sum: T::from(0),
            mask: 0,
        }
    }
    pub fn all(elements: &[T]) -> Self {
        let mask = (1 << elements.len()) - 1;
        let sum = elements.iter().cloned().sum();
        Self { sum, mask }
    }
    pub fn union(left: &Self, right: &Self) -> Self {
        Self {
            sum: left.sum + right.sum,
            mask: left.mask | right.mask,
        }
    }
    pub fn to_vec(&self, elements: &[T]) -> Vec<T> {
        let mut selected_bit = 1;
        let mut vec = Vec::new();
        for x in elements {
            if self.mask & selected_bit > 0 {
                vec.push(*x);
            }
            selected_bit <<= 1;
        }
        vec
    }
}
impl<'a, T: Arith> Subset<T, u64> {
    pub fn elements(&'a self, elements: &'a [T]) -> impl Iterator<Item = T> + 'a {
        elements.iter().enumerate().filter_map(move |(i, x)| {
            if ((1 << i) & self.mask) > 0 {
                Some(*x)
            } else {
                None
            }
        })
    }
}

pub fn all_subsets<T: Arith>(elements: &[T]) -> Option<(Vec<Subset<T, u64>>)> {
    if elements.len() > 63 {
        //TODO: 64 is doable but requires care on the bitshift
        return None;
    }
    let subset_count = 1_u64 << elements.len(); //TODO: dedupe
    Some(
        (0..subset_count)
            .map(|mask| Subset::new(mask, elements))
            .collect(),
    )
}

pub fn split_mask<T: Arith>(mask: u64, elements: &[T]) -> (u64, u64) {
    let mut element_masks = Vec::with_capacity(mask.count_ones() as usize);
    #[allow(clippy::needless_range_loop)]
    for i in 0..64 {
        if (mask & 1 << i) > 0 {
            element_masks.push((elements[i], 1 << i));
        }
    }
    let (smalls, larges) = element_masks.split_at(element_masks.len() / 2);
    let small_mask = smalls
        .iter()
        .fold(0, |acc, (_, element_mask)| acc | element_mask);
    let large_mask = larges
        .iter()
        .fold(0, |acc, (_, element_mask)| acc | element_mask);
    (small_mask, large_mask)
}

#[derive(Debug)]
pub struct Submasks {
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
    /*
    fn size_hint(&self) -> (usize, Option<usize>) {
        let upper = (self.submask as usize).checked_add(1);
        let lower = 1 << self.mask.count_ones();
        (lower, upper)
    }
    */
}

pub fn submasks(mask: u64) -> Submasks {
    Submasks {
        mask,
        submask: mask,
        start: true,
    }
}

#[derive(Debug)]
pub enum Down {}
#[derive(Debug)]
pub enum Up {}

pub trait OrderingDirection {
    fn partial_cmp<T: PartialOrd>(left: &T, right: &T) -> Option<Ordering>;
    fn cmp<T: Ord>(left: &T, right: &T) -> Ordering;
}

impl OrderingDirection for Down {
    fn partial_cmp<T: PartialOrd>(left: &T, right: &T) -> Option<Ordering> {
        left.partial_cmp(right).map(|ordering| ordering.reverse())
    }
    fn cmp<T: Ord>(left: &T, right: &T) -> Ordering {
        left.cmp(right).reverse()
    }
}
impl OrderingDirection for Up {
    fn partial_cmp<T: PartialOrd>(left: &T, right: &T) -> Option<Ordering> {
        left.partial_cmp(right)
    }
    fn cmp<T: Ord>(left: &T, right: &T) -> Ordering {
        left.cmp(right)
    }
}

#[derive(Debug)]
struct HeapPair<T, D> {
    fixed: Subset<T, u64>,
    union: Subset<T, u64>,
    index: usize,
    direction: PhantomData<D>,
}
impl<T: Arith, D> PartialEq for HeapPair<T, D> {
    fn eq(&self, other: &Self) -> bool {
        self.union.sum.eq(&other.union.sum)
    }
}

impl<T: Arith, D> Eq for HeapPair<T, D> {}
//Orderings are reversed since we want a min-heap
impl<T: Arith, D: OrderingDirection> PartialOrd for HeapPair<T, D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <D as OrderingDirection>::partial_cmp(&self.union.sum, &other.union.sum)
            .map(|o| o.reverse())
    }
}
impl<T: Arith, D: OrderingDirection> Ord for HeapPair<T, D> {
    fn cmp(&self, other: &Self) -> Ordering {
        <D as OrderingDirection>::cmp(&self.union.sum, &other.union.sum).reverse()
    }
}

#[derive(Debug)]
pub struct OrderedSubsets<T: Arith, D: OrderingDirection> {
    vec: Vec<Subset<T, u64>>,
    heap: BinaryHeap<HeapPair<T, D>>,
}

pub fn ordered_subsets<T: Arith, D: OrderingDirection>(
    mask: u64,
    elements: &[T],
) -> OrderedSubsets<T, D> {
    let (left_mask, right_mask) = split_mask(mask, elements);
    let mut vec: Vec<Subset<T, u64>> = submasks(left_mask)
        .map(|mask| Subset::new(mask, elements))
        .collect();
    vec.sort_unstable_by(|l, r| <D as OrderingDirection>::cmp(&l.sum, &r.sum));
    let heap: BinaryHeap<HeapPair<T, D>> = submasks(right_mask)
        .map(|mask| {
            let fixed = Subset::new(mask, elements);
            let union = Subset::union(&vec[0], &fixed);
            HeapPair {
                fixed,
                union,
                index: 0,
                direction: PhantomData,
            }
        })
        .collect();
    OrderedSubsets { vec, heap }
}

impl<T: Arith, D: OrderingDirection + Debug> Iterator for OrderedSubsets<T, D> {
    type Item = Subset<T, u64>;
    fn next(&mut self) -> Option<Subset<T, u64>> {
        let mut pair = self.heap.peek_mut()?;
        pair.index += 1;
        match self.vec.get(pair.index) {
            None => Some(PeekMut::pop(pair).union),
            Some(unfixed) => {
                let mut next_union = Subset::union(&unfixed, &pair.fixed);
                swap(&mut pair.union, &mut next_union);
                Some(next_union)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use benchmark_data;
    use proptest::collection::vec;
    use subset::{all_subsets, ordered_subsets, Down, OrderedSubsets, Up};
    proptest! {
        #[test]
        fn prop_ordered_subsets(ref elements in vec(1_i32..100, 1..10)) {
            let mask = (1 << elements.len()) -1;
            let mut expected : Vec<i32> =
                all_subsets(elements).unwrap().into_iter().map(|subset| subset.sum).collect();
            expected.sort();
            let actual_iterator : OrderedSubsets<i32, Up> =
                ordered_subsets(mask, elements);
            let actual : Vec<i32> = actual_iterator.map(|subset| subset.sum).collect();
            assert_eq!(
                expected,
                actual
            );
       }
    }
    proptest! {
        #[test]
        fn prop_ordered_subsets_down(ref elements in vec(1_i32..100, 1..10)) {
            let mask = (1 << elements.len()) -1;
            let mut expected : Vec<i32> =
                all_subsets(elements).unwrap().into_iter().map(|subset| subset.sum).collect();
            expected.sort_by(|l,r| l.cmp(r).reverse());
            let actual_iterator : OrderedSubsets<i32, Down> =
                ordered_subsets(mask, elements);
            let actual : Vec<i32> = actual_iterator.map(|subset| subset.sum).collect();
            assert_eq!(
                expected,
                actual
            );
       }
    }
    #[bench]
    fn bench_ordered_subsets(b: &mut Bencher) {
        let elements = &benchmark_data::SMALL_ELEMENTS;
        let mask = (1 << elements.len()) - 1;
        b.iter(|| ordered_subsets::<i32, Up>(mask, elements).fold(0, |acc, x| acc ^ x.sum));
    }
}
