use arith::Arith;
use ckk::n_kk;
use ess::biased_iterate_subsets_in_range;
use std::cmp;
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
    ub: T,
    min_score: T,
    total_remaining: T,
}

impl<'a, T: Arith> SNP<'a, T> {
    fn snp_helper(&'a mut self) -> Option<T> {
        let range = partition_range(self.ub, self.total_remaining, self.n);
        println!("Called snp helper on:\n{:?}\nrange:{:?}", self, range);
        if self.n == 1 {
            let last_subset = Subset::new(self.mask, self.elements);
            assert!(range.contains(&last_subset.sum));
            let score = cmp::max(self.min_score, last_subset.sum);
            self.current_partitioning.push(last_subset);
            self.best_partitioning.clone_from(self.current_partitioning);
            self.current_partitioning.pop();
            self.current_partitioning.pop();
            return Some(score);
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
        }
        self.current_partitioning.pop();
        return_value
    }
}

#[cfg(test)]
mod tests {
    extern crate cpuprofiler;
    extern crate test;
    use self::test::Bencher;
    use arith::Arith;
    use benchmark_data;
    use ckk::ckk;
    use gcc::find_best_partitioning;
    use proptest::collection::vec;
    use snp::{brute_force, snp};
    use subset::Subset;
    proptest! {
        #[test]
        fn prop_snp_gcc(ref elements in vec(1_i32..100, 1..10), n in (2_u8..5)) {
            let (gcc_results, _) = find_best_partitioning( &elements, n);
            let gcc_score = gcc_results.iter().map(|p| p.sum).max().unwrap();
            let snp_results = snp(&elements, n);
            let snp_score = snp_results[0].sum;
            assert_eq!(snp_score, gcc_score, "SNP got {:?}, GCC got {:?}", snp_results, gcc_results);
       }
    }
    proptest! {
        #[test]
        fn prop_snp_ckk(ref elements in vec(1_i32..100, 1..10)) {
            let ckk_results = ckk(&elements);
            let ckk_score = ckk_results.score();
            let snp_results = snp(&elements, 2);
            let snp_score = snp_results[0].sum;
            assert_eq!(snp_score, ckk_score, "SNP got {:?}, CKK got {:?}", snp_results, ckk_results);
       }
    }
    fn pretty_partitioning<T: Arith>(
        partitions: &[Subset<T, u64>],
        elements: &[T],
    ) -> Vec<(T, Vec<T>)> {
        partitions
            .iter()
            .map(|subset| (subset.sum, subset.to_vec(elements)))
            .collect()
    }
    fn compare_snp_brute(elements: &[i32], n: u8) {
        let brute_results = brute_force(&elements, n);
        let brute_score = brute_results[0].sum;
        let snp_results = snp(&elements, n);
        let snp_score = snp_results.iter().map(|subset| subset.sum).max().unwrap();
        assert_eq!(
            snp_results.len(),
            n as usize,
            "SNP had wrong number of elements. Expected length was {}, got {:?}",
            n,
            pretty_partitioning(&snp_results, elements),
        );
        assert_eq!(
            snp_score,
            brute_score,
            "SNP got {:?}, brute force got {:?}",
            pretty_partitioning(&snp_results, elements),
            pretty_partitioning(&brute_results, elements)
        );
    }
    proptest! {
        #[test]
        fn prop_snp_brute_simple(ref elements in vec(1_i32..6, 1..6)) {
            compare_snp_brute(elements, 2)
       }
    }
    proptest! {
        #[test]
        fn prop_snp_brute(ref elements in vec(1_i32..1000, 1..10)) {
            compare_snp_brute(elements, 4)
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
    #[test]
    fn unit_snp_2() {
        let elements = [5, 5, 4, 4, 3];
        compare_snp_brute(&elements, 2);
    }
    #[bench]
    fn bench_snp(b: &mut Bencher) {
        b.iter(|| snp(&benchmark_data::MEDIUM_ELEMENTS, 4));
    }
}
