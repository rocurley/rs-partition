use arith::Arith;
use ckk::{from_subset, n_kk, KKPartition};
use std::cmp;
use std::collections::BinaryHeap;
use subset::Subset;

#[derive(Debug)]
pub enum RNPResult<T: Arith> {
    TwoWay(KKPartition<T>),
    EvenSplit(Box<RNPResult<T>>, Box<RNPResult<T>>),
    OddSplit(Subset<T, u64>, Box<RNPResult<T>>),
    KKResult(Vec<Subset<T, u64>>),
}
impl<T: Arith> RNPResult<T> {
    pub fn to_vec(&self) -> Vec<Subset<T, u64>> {
        match self {
            RNPResult::TwoWay(kk) => vec![kk.left.clone(), kk.right.clone()],
            RNPResult::EvenSplit(l, r) => {
                let mut v = l.to_vec();
                v.append(&mut r.to_vec());
                v
            }
            RNPResult::OddSplit(first, rest) => {
                let mut v = rest.to_vec();
                v.push(first.clone());
                v
            }
            RNPResult::KKResult(v) => v.clone(),
        }
    }
}

pub fn rnp<T: Arith>(elements: &[T]) -> RNPResult<T> {
    let kk_result = n_kk(elements, 4);
    let mut upper_bound = kk_result.score();
    let mut best = RNPResult::KKResult(kk_result.partitions);
    let heap: BinaryHeap<KKPartition<T>> = elements
        .iter()
        .enumerate()
        .map(|(i, _)| KKPartition::singleton(i, elements))
        .collect();
    rnp_helper(elements, heap, &mut upper_bound, &mut best);
    best
}

fn rnp_helper<T: Arith>(
    elements: &[T],
    mut heap: BinaryHeap<KKPartition<T>>,
    upper_bound: &mut T,
    best: &mut RNPResult<T>,
) {
    let mut first = heap.pop().expect("heap is empty");
    match heap.pop() {
        Some(snd) => {
            //TODO: 2 -> n/2
            if first.score() > T::from(2) * (*upper_bound - 1.into()) {
                return;
            }
            let mut new_heap = heap.clone();
            let mut new_first = first.clone();
            let new_snd = snd.clone();
            new_first = KKPartition::merge(&new_first, &new_snd);
            new_heap.push(new_first);
            rnp_helper(elements, new_heap, upper_bound, best);
            first = KKPartition::merge_rev(&first, &snd);
            heap.push(first);
            rnp_helper(elements, heap, upper_bound, best);
        }
        None => {
            if first.score() > T::from(2) * (*upper_bound - 1.into()) {
                return;
            }
            let left = from_subset(&first.left, elements);
            if left.score() >= *upper_bound {
                return;
            }
            let right = from_subset(&first.right, elements);
            let score = cmp::max(left.score(), right.score());
            if score < *upper_bound {
                *upper_bound = score;
                *best = RNPResult::EvenSplit(
                    Box::new(RNPResult::TwoWay(left)),
                    Box::new(RNPResult::TwoWay(right)),
                );
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
    use rnp::rnp;
    use select::{compare_partitioning_methods, PartitionMethod};
    #[test]
    fn unit_rnp_gcc_small() {
        let elements = [3, 3, 8, 4, 4, 3, 7];
        compare_partitioning_methods(PartitionMethod::RNP, PartitionMethod::GCC, &elements, 4);
    }
    proptest! {
        #[test]
        fn prop_rnp_gcc_small(ref elements in vec(1_i32..10, 1..8)) {
            compare_partitioning_methods(PartitionMethod::RNP, PartitionMethod::GCC, &elements, 4);
       }
    }
    proptest! {
        #[test]
        fn prop_rnp_gcc(ref elements in vec(1_i32..100, 1..10)) {
            compare_partitioning_methods(PartitionMethod::RNP, PartitionMethod::GCC, &elements, 4);
       }
    }
    #[bench]
    fn bench_rnp(b: &mut Bencher) {
        b.iter(|| rnp(&benchmark_data::MEDIUM_ELEMENTS));
    }
}
