#[path = "arith.rs"]
pub mod arith;
use self::arith::Arith;
use std::ops::RangeInclusive;

#[derive(Debug, PartialEq, Eq)]
struct Subset<T, M> {
    sum: T,
    mask: M,
}
impl<T: Arith> Subset<T, u64> {
    fn new(mask: u64, elements: &[T]) -> Self {
        let mut selected_bit = 1;
        let mut sum = T::from(0);
        for x in elements {
            if mask & selected_bit > 0 {
                sum += *x;
            }
            selected_bit <<= 1;
        }
        Subset { sum, mask }
    }
}

fn all_subsets<T: Arith>(elements: &[T]) -> Option<(Vec<Subset<T, u64>>)> {
    if elements.len() > 63 {
        //TODO: 64 is doable but requires care on the bitshift
        return None;
    }
    let subset_count = 1u64 << elements.len(); //TODO: dedupe
    Some(
        (0..subset_count)
            .map(|mask| Subset::new(mask, elements))
            .collect(),
    )
}

fn naive_subsets_in_range<T: Arith>(
    elements: &[T],
    range: RangeInclusive<T>,
) -> Option<(Vec<Subset<T, u64>>)> {
    let mut subsets = all_subsets(elements)?;
    subsets.retain(|subset| range.contains(&subset.sum));
    Some(subsets)
}

struct Submasks {
    mask: u64,
    submask: u64,
}
impl Iterator for Submasks where {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        if self.mask == 0 {
            return None;
        }
        if self.submask == 0 {
            self.mask = 0;
            return Some(self.submask);
        }
        let to_return = self.submask;
        self.submask -= 1;
        self.submask &= self.mask;
        Some(to_return)
    }
}

fn submasks(mask: u64) -> Submasks {
    Submasks {
        mask,
        submask: mask,
    }
}

#[cfg(test)]
mod tests {
    use ehs::{all_subsets, submasks, Subset};
    #[test]
    fn unit_all_subsets() {
        let elements = vec![1, 2, 3];
        let subsets = all_subsets(&elements).unwrap();
        let expected = vec![
            Subset { mask: 0, sum: 0 },
            Subset { mask: 1, sum: 1 },
            Subset { mask: 2, sum: 2 },
            Subset { mask: 3, sum: 3 },
            Subset { mask: 4, sum: 3 },
            Subset { mask: 5, sum: 4 },
            Subset { mask: 6, sum: 5 },
            Subset { mask: 7, sum: 6 },
        ];
        assert_eq!(subsets, expected);
    }
    #[test]
    fn unit_submasks() {
        let mask = 0b10101u64;
        let mut expected = vec![
            0b00000u64, 0b00001u64, 0b00100u64, 0b00101u64, 0b10000u64, 0b10001u64, 0b10100u64,
            0b10101u64,
        ];
        expected.reverse();
        let submasks_iterator = submasks(mask);
        let actual: Vec<u64> = submasks_iterator.collect();
        assert_eq!(actual, expected);
    }
}
