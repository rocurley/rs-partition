extern crate cpuprofiler;
extern crate num;

use super::arith::Arith;
use num::{one, zero};
use std::iter::Iterator;

#[derive(Clone)]
pub struct Partition<T: Arith> {
    sum: T,
    length: usize,
    elements: Box<[T]>,
}

impl<T: Arith> Partition<T> {
    fn new(capacity: usize) -> Self {
        Partition {
            sum: zero(),
            length: 0,
            elements: vec![zero(); capacity].into_boxed_slice(),
        }
    }
    fn push(&mut self, x: T) {
        self.sum += x;
        *&mut self.elements[self.length] = x;
        self.length += 1;
    }
    fn pop(&mut self) {
        self.length -= 1;
        self.sum -= *&self.elements[self.length];
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
    partitions: &mut [Partition<T>],
    current_best: &mut (Vec<Partition<T>>, T),
    constants: Constants<T>,
) {
    if elements.len() == 0 {
        consider_partitioning(current_best, partitions);
        return;
    }
    let largest_sum = partitions
        .iter()
        .map(|partition| partition.sum)
        .max()
        .expect("partitions is empty");
    if largest_sum * constants.n_partitions - constants.total
        >= (*current_best).1 * (constants.n_partitions - one())
    {
        return;
    }
    let x = elements[0];
    let (min_index, mut last_sum): (usize, T) = partitions
        .iter()
        .map(|partition| partition.sum)
        .enumerate()
        .min_by_key(|&(_, sum)| sum)
        .expect("partitions is empty");
    partitions[min_index].push(x);
    expand_partitions(&elements[1..], partitions, current_best, constants);
    partitions[min_index].pop();
    while let Some((i, sum)) = partitions
        .iter()
        .map(|partition| partition.sum)
        .filter(|&sum| sum > last_sum)
        .enumerate()
        .min_by_key(|&(_, sum)| sum)
    {
        last_sum = sum;
        partitions[i].push(x);
        expand_partitions(&elements[1..], partitions, current_best, constants);
        partitions[i].pop();
    }
}

fn score_partitioning<T: Arith>(partitions: &[Partition<T>]) -> T {
    let mut max_sum = partitions[0].sum;
    let mut min_sum = max_sum;
    for partition in partitions[1..].into_iter() {
        if partition.sum > max_sum {
            max_sum = partition.sum;
        } else if partition.sum < min_sum {
            min_sum = partition.sum;
        }
    }
    max_sum - min_sum
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
        total: elements.iter().map(|x| x.clone()).sum(),
        n_partitions: n_partitions.into(),
    };
    expand_partitions(
        elements,
        partitions.as_mut_slice(),
        &mut scored_best_partitioning,
        constants,
    );
    scored_best_partitioning
}
