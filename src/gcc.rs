extern crate cpuprofiler;
extern crate num;

use super::arith::Arith;
use num::zero;
use std::iter::Iterator;

#[derive(Clone, Debug)]
pub struct Partition<T: Arith> {
    sum: T,
    length: usize,
    elements: Box<[T]>,
}

impl<T: Arith> Partition<T> {
    fn new(capacity: usize) -> Self {
        Self {
            sum: zero(),
            length: 0,
            elements: vec![zero(); capacity].into_boxed_slice(),
        }
    }
    fn push(&mut self, x: T) {
        self.sum += x;
        self.elements[self.length] = x;
        self.length += 1;
    }
    fn pop(&mut self) {
        self.length -= 1;
        self.sum -= self.elements[self.length];
    }
    pub fn print(&self) {
        println!("{:?} : {:?}", self.sum, &self.elements[0..self.length]);
    }
    pub fn to_vec(&self) -> Vec<T> {
        self.elements[0..self.length].to_vec()
    }
    pub fn sum(&self) -> T {
        self.sum
    }
}

fn consider_partitioning<T: Arith>(
    current_best: &mut (Vec<Partition<T>>, T),
    candidate: &[Partition<T>],
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
    partitions: &mut [Partition<T>],
    current_best: &mut (Vec<Partition<T>>, T),
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
    let x = elements[index];
    let mut ordered_indexed_partition_sums: Vec<(usize, T)> = partitions
        .iter()
        .map(|partition| partition.sum)
        .enumerate()
        .collect();
    ordered_indexed_partition_sums.sort_by_key(|&(_, sum)| sum);
    for (i, _) in ordered_indexed_partition_sums {
        partitions[i].push(x);
        expand_partitions(&elements, index + 1, partitions, current_best, constants);
        partitions[i].pop();
        if largest_sum == (*current_best).1 {
            return;
        }
    }
}

fn score_partitioning<T: Arith>(partitions: &[Partition<T>]) -> T {
    partitions
        .iter()
        .map(|partition| partition.sum)
        .max()
        .expect("partitions is empty")
}

pub fn find_best_partitioning<T: Arith>(
    n_partitions: u8,
    elements: &[T],
) -> (Vec<Partition<T>>, T) {
    let mut partitions: Vec<Partition<T>> = (0..n_partitions)
        .map(|_| Partition::new(elements.len()))
        .collect();
    let mut best_partitioning: Vec<Partition<T>> = (0..n_partitions)
        .map(|_| Partition::new(elements.len()))
        .collect();
    for el in elements.iter() {
        best_partitioning[0].push(el.clone());
    }
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
