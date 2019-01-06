use arith::Arith;
use ckk::n_kk;
use ess::iterate_subsets_in_range;
use std::mem::swap;
use subset::Subset;

pub fn snp<T: Arith>(elements: &[T], n: u8) -> Vec<Subset<T, u64>> {
    let mask = (1 << n) - 1;
    let mut current_partitioning = Vec::new();
    let mut best_partitioning = n_kk(elements, n as usize).partitions;
    let current_best = best_partitioning[0].sum;
    let total = best_partitioning.iter().map(|subset| subset.sum).sum();
    let mut snp = SNP {
        elements,
        n,
        mask,
        current_partitioning: &mut current_partitioning,
        best_partitioning: &mut best_partitioning,
        current_best,
        total,
    };
    snp.snp_helper();
    return best_partitioning;
}

struct SNP<'a, T> {
    elements: &'a [T],
    n: u8,
    mask: u64,
    current_partitioning: &'a mut Vec<Subset<T, u64>>,
    best_partitioning: &'a mut Vec<Subset<T, u64>>,
    current_best: T,
    total_remaining: T,
}

impl<'a, T: Arith> SNP<'a, T> {
    fn snp_helper(&'a mut self) {
        let ub = self
            .current_partitioning
            .last()
            .map_or(self.current_best, |subset| subset.sum);
        //TODO: there are other lower bounds available.
        let lb = T::from(1) + (self.total_remaining - T::from(1)) / T::from(self.n);
        let range = lb..self.current_best;
        let mut subsets_iter = iterate_subsets_in_range(self.mask, self.elements, range);
        if self.n == 1 {
            unimplemented!();
        }
        while let Some(first_subset) = subsets_iter.next() {
            let mask = self.mask ^ first_subset.mask;
            let total_remaining = self.total_remaining - first_subset.sum;
            self.current_partitioning.push(first_subset);
            let mut child = SNP {
                elements: self.elements,
                n: self.n - 1,
                mask,
                current_partitioning: self.current_partitioning,
                best_partitioning: self.best_partitioning,
                current_best: self.current_best,
                total_remaining,
            };
            child.snp_helper();
            self.current_best = child.current_best;
        }
    }
}
