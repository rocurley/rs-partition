use arith::Arith;
use std::iter::{empty, once};
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

pub fn partition<T: Arith>(elements: &[T], n: u8) -> Vec<Subset<T, u64>> {
    let mask = (1 << elements.len()) - 1;
    let total = elements.iter().fold(T::from(0), |acc, &x| acc + x);
    let mut out = all_partitions(mask, elements, n, total)
        .min_by_key(|partitioning| Some(partitioning.last()?.sum))
        .unwrap();
    out.reverse();
    out
}
