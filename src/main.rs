extern crate num;

use std::ops::{AddAssign, SubAssign, Sub};
use num::{Zero, zero};
use std::iter::{repeat, Iterator};
use std::io::stdin;
use std::env::args;

trait Arith : AddAssign + SubAssign + Sub<Output=Self> + Copy + PartialOrd + Ord + Zero {}
impl<T> Arith for T
    where T :  AddAssign + SubAssign + Sub<Output=T> + Copy + PartialOrd + Ord + Zero {}

fn main() {
    let string_args :Vec<String> = args().collect();
    let n_partitions = string_args[1].parse().expect("Couldn't parse argument");
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Couldn't read from stdin");
    let elements_result : Result<Vec<f64>, std::num::ParseFloatError> = input.trim().split(",").map(|i| i.parse()).collect();
    let mut elements : Vec<i32> = elements_result.expect("Couldn't parse input").into_iter().map(|n| (n * 1000000.) as i32).collect();
    elements.sort();
    let (partitions, score) = find_best_partitioning(n_partitions, &elements);
    println!("Score: {}", score);
    for partition in partitions {
        println!("{} : {:?}", partition.sum, partition.elements);
    }
}

#[derive(Clone)]
struct Partition<T  : Arith> {
    sum : T,
    elements : Vec<T>
}

impl<T : Arith> Partition<T> {
    fn new(capacity : usize) -> Self {
        Partition{
            sum : zero(),
            elements : Vec::with_capacity(capacity)
        }
    }
    fn push(&mut self, x : T) {
        self.sum += x;
        self.elements.push(x);
    }
    fn pop(&mut self) -> T {
        let x = self.elements.pop().expect("Popped empty partition");
        self.sum  -= x;
        x
    }
}

fn expand_partitions<T, F>(yield_fn : &mut F, elements : &[T], partitions : &mut[Partition<T>])
    where T : Arith,
    F : FnMut(&[Partition<T>])
{
    if elements.len() == 0 {
        yield_fn(partitions);
        return;
    }
    let x = elements[0];
    'outer: for i in 0..partitions.len() {
        for j in 0..i {
            if partitions[i].sum == partitions[j].sum {
                continue 'outer;
            }
        }
        partitions[i].push(x);
        expand_partitions(yield_fn, &elements[1..], partitions);
        partitions[i].pop();
    }
}

fn score_partitioning<T : Arith>(partitions : &[Partition<T>]) -> T {
    let mut max_sum = partitions[0].sum;
    let mut min_sum = max_sum;
    for partition in partitions[1..].into_iter(){
        if partition.sum > max_sum {
            max_sum = partition.sum;
        } else if partition.sum < min_sum {
            min_sum = partition.sum;
        }
    }
    max_sum - min_sum
}

fn find_best_partitioning<T : Arith>(n_partitions : usize, elements : &[T]) -> (Vec<Partition<T>>, T) {
    let mut partitions : Vec<Partition<T>> = repeat(Partition::new(elements.len())).take(n_partitions).collect();
    let mut best_partitioning : Option<(Vec<Partition<T>>, T)> = None;
    { //scope for consider_partitioning
        let mut consider_partitioning = |partitioning : &[Partition<T>]| {
            let score = score_partitioning(partitioning);
            match best_partitioning {
                None => best_partitioning = Some((partitioning.to_vec(), score)),
                Some((ref mut current_partitioning, ref mut current_score)) => {
                    if *current_score > score {
                        *current_score = score;
                        current_partitioning.clone_from_slice(partitioning)
                    }
                },
            }
        };
        expand_partitions(&mut consider_partitioning, elements, partitions.as_mut_slice());
    }
    best_partitioning.expect("Considered 0 partitions")
}
