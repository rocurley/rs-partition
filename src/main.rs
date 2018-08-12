extern crate cpuprofiler;
extern crate num;

use cpuprofiler::PROFILER;
use num::{one, zero, Integer};
use std::convert::From;
use std::env::args;
use std::io::stdin;
use std::iter::{Iterator, Sum};
use std::ops::{AddAssign, SubAssign};

trait Arith: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum {}
impl<T> Arith for T where T: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum {}

fn main() {
    let string_args: Vec<String> = args().collect();
    let n_partitions = string_args[1].parse().expect("Couldn't parse argument");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Couldn't read from stdin");
    let elements_result: Result<Vec<f64>, std::num::ParseFloatError> =
        input.trim().split(",").map(|i| i.parse()).collect();
    let mut elements: Vec<i32> = elements_result
        .expect("Couldn't parse input")
        .into_iter()
        .map(|n| (n * 1000000.) as i32)
        .collect();
    elements.sort_by_key(|x| -x);
    PROFILER
        .lock()
        .unwrap()
        .start("./my-prof.profile")
        .expect("Couldn't start");
    let (partitions, score) = find_best_partitioning(n_partitions, &elements);
    PROFILER.lock().unwrap().stop().expect("Couldn't stop");
    println!("Score: {}", score);
    for partition in partitions {
        println!("{} : {:?}", partition.sum, partition.elements);
    }
}

#[derive(Clone)]
struct Partition<T: Arith> {
    sum: T,
    elements: Vec<T>,
}

impl<T: Arith> Partition<T> {
    fn new(capacity: usize) -> Self {
        Partition {
            sum: zero(),
            elements: Vec::with_capacity(capacity),
        }
    }
    fn push(&mut self, x: T) {
        self.sum += x;
        self.elements.push(x);
    }
    fn pop(&mut self) -> T {
        let x = self.elements.pop().expect("Popped empty partition");
        self.sum -= x;
        x
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

fn find_best_partitioning<T: Arith>(n_partitions: u8, elements: &[T]) -> (Vec<Partition<T>>, T) {
    let mut partitions: Vec<Partition<T>> = (0..n_partitions)
        .map(|_| Partition::new(elements.len()))
        .collect();
    let mut best_partitioning: Vec<Partition<T>> = partitions.clone();
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
