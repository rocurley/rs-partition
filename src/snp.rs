use arith::Arith;
use ckk;
use ckk::n_kk;
use ess::biased_iterate_subsets_in_range;
use std::cmp;
use std::ops::Range;
use subset::Subset;

pub fn snp<T: Arith>(elements: &[T], n: u8) -> Vec<Subset<T, u64>> {
    let mask = (1 << elements.len()) - 1;
    let mut best_partitioning = n_kk(elements, n).partitions;
    let mut current_partitioning = Vec::new();
    let ub = best_partitioning[0].sum;
    let total_remaining = best_partitioning.iter().map(|subset| subset.sum).sum();
    let min_score = (total_remaining - 1.into()) / n.into() + 1.into();
    let mut snp = SNP {
        elements,
        n,
        mask,
        current_partitioning: &mut current_partitioning,
        best_partitioning: &mut best_partitioning,
        ub,
        min_score,
        total_remaining,
    };
    snp.snp_helper();
    best_partitioning
}

fn partition_range<T: Arith>(ub: T, total: T, n: u8) -> Range<T> {
    //TODO: are there are other lower bounds available?
    let lb = total - T::from(n - 1) * (ub - 1.into());
    lb..ub
}

#[derive(Debug)]
struct SNP<'a, T> {
    elements: &'a [T],
    n: u8,
    mask: u64,
    current_partitioning: &'a mut Vec<Subset<T, u64>>,
    best_partitioning: &'a mut Vec<Subset<T, u64>>,
    //Upper bound on score that we're interested in. Corresponds to the best partitioning seen
    //so far: if you can't beat the best so far, why bother? Seeded with n_kk.
    ub: T,
    //Absolute best score that this branch can achieve. This starts with a perfect partitioning,
    //but can be raised if prior passes paritioned off something with a higher sum. Achieving
    //or beating this immediately terminate the branch, since further improvement would either
    //be useless (in the case of higher-sum prior passes) or impossible (perfect partitioning).
    min_score: T,
    total_remaining: T,
}

impl<'a, T: Arith> SNP<'a, T> {
    fn snp_helper(&'a mut self) -> Option<T> {
        let range = partition_range(self.ub, self.total_remaining, self.n);
        if self.n == 1 {
            let last_subset = Subset::new(self.mask, self.elements);
            assert!(range.contains(&last_subset.sum));
            let score = cmp::max(self.min_score, last_subset.sum);
            self.best_partitioning.clone_from(self.current_partitioning);
            self.best_partitioning.push(last_subset);
            return Some(score);
        }
        if self.n == 2 && self.mask.count_ones() < 12 {
            let masked_subset = Subset {
                mask: self.mask,
                sum: self.total_remaining,
            };
            let partitioning = ckk::from_subset(&masked_subset, self.elements);
            let score = partitioning.score();
            if score >= self.ub {
                return None;
            }
            self.best_partitioning
                .clone_from(&self.current_partitioning);
            self.best_partitioning.push(partitioning.left);
            self.best_partitioning.push(partitioning.right);
            return Some(cmp::max(self.min_score, score));
        }
        let mut subsets_iterator = biased_iterate_subsets_in_range(self.mask, self.elements, range);
        let mut return_value = None;
        while let Some(first_subset) = subsets_iterator.next() {
            let mask = self.mask ^ first_subset.mask;
            let total_remaining = self.total_remaining - first_subset.sum;
            let min_score = cmp::max(self.min_score, first_subset.sum);
            self.current_partitioning.push(first_subset);
            let mut child = SNP {
                elements: self.elements,
                n: self.n - 1,
                mask,
                current_partitioning: self.current_partitioning,
                best_partitioning: self.best_partitioning,
                ub: self.ub,
                min_score,
                total_remaining,
            };
            if let Some(new_best) = child.snp_helper() {
                if new_best <= self.min_score {
                    self.current_partitioning.pop();
                    return Some(self.min_score);
                }
                return_value = Some(new_best);
                self.ub = new_best;
                subsets_iterator.restrict_range(partition_range(
                    self.ub,
                    self.total_remaining,
                    self.n,
                ));
            }
            self.current_partitioning.pop();
        }
        return_value
    }
}

#[cfg(test)]
mod tests {
    extern crate cpuprofiler;
    extern crate test;
    use self::test::Bencher;
    use benchmark_data;
    use proptest::collection::vec;
    use select::{compare_partitioning_methods, PartitionMethod};
    use snp::snp;
    use subset::Subset;
    proptest! {
        #[test]
        fn prop_snp_gcc(ref elements in vec(1_i32..100, 1..13), n in (2_u8..5)) {
            compare_partitioning_methods(PartitionMethod::GCC, PartitionMethod::SNP, &elements, n);
       }
    }
    proptest! {
        #[test]
        fn prop_snp_ckk(ref elements in vec(1_i32..100, 1..10)) {
            compare_partitioning_methods(PartitionMethod::CKK, PartitionMethod::SNP, &elements, 2);
       }
    }
    proptest! {
        #[test]
        fn prop_snp_brute_simple(ref elements in vec(1_i32..6, 1..6)) {
            compare_partitioning_methods(PartitionMethod::Brute, PartitionMethod::SNP, &elements, 2);
       }
    }
    proptest! {
        #[test]
        fn prop_snp_brute(ref elements in vec(1_i32..1000, 1..10)) {
            compare_partitioning_methods(PartitionMethod::Brute, PartitionMethod::SNP, &elements, 4);
       }
    }
    #[test]
    fn unit_snp() {
        let elements = [24, 17, 24, 25, 25];
        let snp_results = snp(&elements, 2);
        let expected = vec![
            Subset::new(0b00111, &elements),
            Subset::new(0b11000, &elements),
        ];
        assert_eq!(snp_results, expected);
    }
    #[bench]
    fn bench_snp(b: &mut Bencher) {
        b.iter(|| snp(&benchmark_data::MEDIUM_ELEMENTS, 4));
    }
}
