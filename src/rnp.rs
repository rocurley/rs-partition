use arith::Arith;
use ckk::{from_subset, n_kk, KKPartition};
use std::collections::BinaryHeap;
use subset::Subset;

#[derive(Debug)]
pub enum RNPResult<T: Arith> {
    TwoWay(KKPartition<T>),
    EvenSplit(Box<RNPResult<T>>, Box<RNPResult<T>>),
    OddSplit(Subset<T, u64>, Box<RNPResult<T>>),
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
        }
    }
}

pub fn rnp<T: Arith>(elements: &[T]) -> RNPResult<T> {
    let mut upper_bound = n_kk(elements, 4).score();
    let mut best = None;
    let heap: BinaryHeap<KKPartition<T>> = elements
        .iter()
        .enumerate()
        .map(|(i, _)| KKPartition::singleton(i, elements))
        .collect();
    rnp_helper(elements, heap, &mut upper_bound, &mut best);
    best.expect("KK heursitic was optimal, which isn't properly handled yet :(")
}

fn rnp_helper<T: Arith>(
    elements: &[T],
    mut heap: BinaryHeap<KKPartition<T>>,
    upper_bound: &mut T,
    best: &mut Option<RNPResult<T>>,
) {
    let mut first = heap.pop().expect("heap is empty");
    match heap.pop() {
        None => {
            if first.score() / 2.into() < *upper_bound {
                let left = from_subset(&first.left, elements);
                if (left.score() + first.score()) / 2.into() < *upper_bound {
                    let right = from_subset(&first.right, elements);
                    let score = (first.score() + right.score() + left.score()) / 2.into();
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
            let rest_score = snd.score() + heap.iter().map(|p| p.score()).sum();
            if (first.score() > rest_score)
                && ((first.score() - rest_score) / 2.into() > *upper_bound)
            {
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
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use gcc::find_best_partitioning;
    use proptest::collection::vec;
    use rnp::rnp;
    proptest! {
        #[test]
        fn prop_rnp_gcc(ref elements in vec(1_i32..100, 1..10)) {
            let (gcc_results, _) = find_best_partitioning(4, &elements);
            let gcc_sums : Vec<i32> = gcc_results.to_vec().into_iter().map(|p| p.sum).collect();
            let gcc_score = gcc_sums.iter().max().unwrap() - gcc_sums.iter().min().unwrap();
            let rnp_results = rnp(&elements);
            let rnp_sums : Vec<i32> = rnp_results.to_vec().into_iter().map(|p| p.sum).collect();
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
    fn bench_rnp(b: &mut Bencher) {
        #[allow(clippy::unreadable_literal)]
        let elements = vec![
            403188, 4114168, 4114168, 5759835, 5759835, 5759835, 2879917, 8228336, 8228336,
            8228336, 8228336, 8228336, 8228336, 8228336, 2057084, 2057084, 2057084, 2057084,
            537584, 537584, 537584,
        ];
        b.iter(|| rnp(&elements));
    }
}
