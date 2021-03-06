extern crate cpuprofiler;
extern crate num;

use super::arith::Arith;
use std::iter::Iterator;
use std::mem;
use subset::Subset;

fn consider_partitioning<T: Arith>(
    current_best: &mut (Vec<Subset<T, u64>>, T),
    candidate: &[Subset<T, u64>],
) {
    let score = score_partitioning(candidate);
    let (ref mut current_partitioning, ref mut current_score) = current_best;
    if *current_score > score {
        *current_score = score;
        current_partitioning.clone_from_slice(candidate)
    }
}

#[derive(Clone, Copy)]
struct Constants<T: Arith> {
    total: T,
    n_partitions: T,
}

fn expand_partitions<T: Arith>(
    elements: &[T],
    index: usize,
    partitions: &mut [Subset<T, u64>],
    current_best: &mut (Vec<Subset<T, u64>>, T),
    constants: Constants<T>,
) {
    if elements.len() <= index {
        consider_partitioning(current_best, partitions);
        return;
    }
    let largest_sum = score_partitioning(partitions);
    if largest_sum >= (*current_best).1 {
        return;
    }
    let mut ordered_indexed_partition_sums: Vec<(usize, T)> = partitions
        .iter()
        .map(|partition| partition.sum)
        .enumerate()
        .collect();
    ordered_indexed_partition_sums.sort_by_key(|&(_, sum)| sum);
    for (i, _) in ordered_indexed_partition_sums {
        let mut saved_subset = Subset::union(&partitions[i], &Subset::from_index(index, elements));
        mem::swap(&mut saved_subset, &mut partitions[i]);
        expand_partitions(&elements, index + 1, partitions, current_best, constants);
        partitions[i] = saved_subset;
        if largest_sum == (*current_best).1 {
            return;
        }
    }
}

fn score_partitioning<T: Arith>(partitions: &[Subset<T, u64>]) -> T {
    partitions
        .iter()
        .map(|partition| partition.sum)
        .max()
        .expect("partitions is empty")
}

pub fn find_best_partitioning<T: Arith>(
    elements: &[T],
    n_partitions: u8,
) -> (Vec<Subset<T, u64>>, T) {
    let mut partitions: Vec<Subset<T, u64>> = vec![Subset::empty(); n_partitions as usize];
    let mut best_partitioning = partitions.clone();
    best_partitioning[0] = Subset::all(elements);
    let score = score_partitioning(&best_partitioning);
    let mut scored_best_partitioning = (best_partitioning, score);
    let constants = Constants {
        total: elements.iter().cloned().sum(),
        n_partitions: n_partitions.into(),
    };
    expand_partitions(
        elements,
        0,
        partitions.as_mut_slice(),
        &mut scored_best_partitioning,
        constants,
    );
    scored_best_partitioning
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use benchmark_data;
    use gcc::find_best_partitioning;
    use proptest::collection::vec;
    use select::{compare_partitioning_methods, PartitionMethod};
    #[bench]
    fn bench_gcc(b: &mut Bencher) {
        b.iter(|| find_best_partitioning(&benchmark_data::SMALL_ELEMENTS, 4));
    }
    proptest! {
        #[test]
        fn prop_gcc_brute(ref elements in vec(1_i32..1000, 1..10)) {
            compare_partitioning_methods(PartitionMethod::Brute, PartitionMethod::GCC, &elements, 4);
       }
    }
}
