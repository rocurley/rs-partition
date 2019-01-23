use arith::Arith;
use ckk::n_kk;
use ess::iterate_subsets_in_range;
use std::iter::{empty, once};
use std::ops::Range;
use subset::{submasks, Subset};

fn all_partitions<'a, T: Arith>(
    mask: u64,
    elements: &'a [T],
    n: u8,
    max: T,
) -> Box<Iterator<Item = Vec<Subset<T, u64>>> + 'a> {
    if n == 1 {
        let subset = Subset::new(mask, elements);
        if subset.sum <= max {
            Box::new(once(vec![subset]))
        } else {
            Box::new(empty())
        }
    } else {
        Box::new(
            submasks(mask)
                .filter_map(move |submask| {
                    let subset = Subset::new(submask, elements);
                    if subset.sum <= max {
                        Some(subset)
                    } else {
                        None
                    }
                })
                .flat_map(move |subset| {
                    all_partitions(mask ^ subset.mask, elements, n - 1, subset.sum).map(
                        move |mut rest| {
                            rest.push(subset.clone());
                            rest
                        },
                    )
                }),
        )
    }
}

pub fn brute_force<T: Arith>(elements: &[T], n: u8) -> Vec<Subset<T, u64>> {
    let mask = (1 << elements.len()) - 1;
    let total = elements.iter().fold(T::from(0), |acc, &x| acc + x);
    let mut out = all_partitions(mask, elements, n, total)
        .min_by_key(|partitioning| Some(partitioning.last()?.sum))
        .unwrap();
    out.reverse();
    out
}

pub fn snp<T: Arith>(elements: &[T], n: u8) -> Vec<Subset<T, u64>> {
    let base_mask = (1 << elements.len()) - 1;
    let mut best_partitioning = n_kk(elements, n as usize).partitions;
    let mut ub = best_partitioning[0].sum;
    let total = best_partitioning.iter().map(|subset| subset.sum).sum();
    let range = partition_range(ub, total, n);
    let mut subsets_iterator = iterate_subsets_in_range(base_mask, elements, range);
    while let Some(first_subset) = subsets_iterator.next() {
        let mask = base_mask ^ first_subset.mask;
        let total_remaining = total - first_subset.sum;
        let child_ub = first_subset.sum + T::from(1);
        let mut current_partitioning = vec![first_subset];
        let mut snp = SNP {
            elements,
            n: n - 1,
            mask,
            current_partitioning: &mut current_partitioning,
            ub: child_ub,
            total_remaining,
        };
        if snp.snp_helper() {
            best_partitioning = current_partitioning.clone();
            ub = current_partitioning[0].sum;
            subsets_iterator.range = partition_range(ub, total, n);
        }
    }
    best_partitioning
}

fn partition_range<T: Arith>(ub: T, total: T, n: u8) -> Range<T> {
    //TODO: there are other lower bounds available.
    let lb = T::from(1) + (total - T::from(1)) / T::from(n);
    lb..ub
}

#[derive(Debug)]
struct SNP<'a, T> {
    elements: &'a [T],
    n: u8,
    mask: u64,
    current_partitioning: &'a mut Vec<Subset<T, u64>>,
    ub: T,
    total_remaining: T,
}

impl<'a, T: Arith> SNP<'a, T> {
    fn snp_helper(&'a mut self) -> bool {
        let range = partition_range(self.ub, self.total_remaining, self.n);
        if self.n == 1 {
            let last_subset = Subset::new(self.mask, self.elements);
            assert!(range.contains(&last_subset.sum));
            self.current_partitioning.push(last_subset);
            return true;
        }
        for first_subset in iterate_subsets_in_range(self.mask, self.elements, range) {
            let mask = self.mask ^ first_subset.mask;
            let total_remaining = self.total_remaining - first_subset.sum;
            //TODO: does the trait I chose guarentee that +1 is successor?
            let ub = first_subset.sum + T::from(1);
            self.current_partitioning.push(first_subset);
            let mut child = SNP {
                elements: self.elements,
                n: self.n - 1,
                mask,
                current_partitioning: self.current_partitioning,
                ub,
                total_remaining,
            };
            if child.snp_helper() {
                return true;
            }
            self.current_partitioning.pop();
        }
        false
    }
}

#[cfg(test)]
mod tests {
    extern crate cpuprofiler;
    extern crate test;
    use self::cpuprofiler::PROFILER;
    use self::test::Bencher;
    use ckk::ckk;
    use gcc::find_best_partitioning;
    use proptest::collection::vec;
    use snp::{brute_force, snp};
    use subset::Subset;
    proptest! {
        #[test]
        fn prop_snp_gcc(ref elements in vec(1_i32..100, 1..10), n in (2_u8..5)) {
            let (gcc_results, _) = find_best_partitioning(n, &elements);
            let gcc_sums : Vec<i32> = gcc_results.to_vec().into_iter().map(|p| p.sum).collect();
            let gcc_score = *gcc_sums.iter().max().unwrap();
            let snp_results = snp(&elements, n);
            let snp_score = snp_results[0].sum;
            assert_eq!(snp_score, gcc_score, "SNP got {:?}, GCC got {:?}", snp_results, gcc_results);
       }
    }
    proptest! {
        #[test]
        fn prop_snp_ckk(ref elements in vec(1_i32..100, 1..10)) {
            let ckk_results = ckk(&elements);
            let ckk_score = ckk_results.new_score();
            let snp_results = snp(&elements, 2);
            let snp_score = snp_results[0].sum;
            assert_eq!(snp_score, ckk_score, "SNP got {:?}, CKK got {:?}", snp_results, ckk_results);
       }
    }
    proptest! {
        #[test]
        fn prop_snp_brute(ref elements in vec(1_i32..100, 1..10)) {
            let brute_results = brute_force(&elements, 4);
            let brute_score = brute_results[0].sum;
            let snp_results = snp(&elements, 4);
            let snp_score = snp_results[0].sum;
            assert_eq!(snp_score, brute_score, "SNP got {:?}, brute force got {:?}", snp_results, brute_results);
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
        #[allow(clippy::unreadable_literal)]
        let elements = vec![
            403188, 4114168, 4114168, 5759835, 5759835, 5759835, 2879917, 8228336, 8228336,
            8228336, 8228336, 8228336, 8228336, 8228336, 2057084, 2057084, 2057084, 2057084,
            537584, 537584, 537584,
        ];
        PROFILER.lock().unwrap().start("snp.profile").unwrap();
        b.iter(|| snp(&elements, 4));
        PROFILER.lock().unwrap().stop().unwrap();
    }
}
