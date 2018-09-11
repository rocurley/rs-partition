use num::Integer;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::convert::From;
use std::fmt::{Debug, Display};
use std::iter::{Iterator, Sum};
use std::ops::{AddAssign, SubAssign};

pub trait Arith:
    Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display
{
}
impl<T> Arith for T where
    T: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display
{}

#[derive(Eq, Debug)]
pub struct KKPartition<T: Arith> {
    left: Vec<T>,
    right: Vec<T>,
    score: T,
}

impl<T: Arith> PartialEq for KKPartition<T> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
impl<T: Arith> PartialOrd for KKPartition<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.score.cmp(&other.score))
    }
}
impl<T: Arith> Ord for KKPartition<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl<T: Arith> KKPartition<T> {
    fn merge(&mut self, mut other: Self) {
        self.left.append(&mut other.right);
        self.right.append(&mut other.left);
        self.score -= other.score;
    }
    fn merge_rev(&mut self, mut other: Self) {
        self.left.append(&mut other.left);
        self.right.append(&mut other.right);
        self.score += other.score;
    }

    fn singleton(x: T) -> Self {
        KKPartition {
            left: vec![x],
            right: Vec::new(),
            score: x,
        }
    }
}

pub fn kk<T: Arith>(elements: &[T]) -> KKPartition<T> {
    let mut heap: BinaryHeap<KKPartition<T>> = elements
        .into_iter()
        .map(|&x| KKPartition::singleton(x))
        .collect();
    loop {
        match (heap.pop(), heap.pop()) {
            (None, None) => panic!("heap is empty"),
            (None, Some(_)) => panic!("first empty, snd not"),
            (Some(first), None) => return first,
            (Some(mut first), Some(snd)) => {
                first.merge(snd);
                heap.push(first);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Diff,
    Sum,
    Fill,
}

fn reconstruct_ckk<T: Arith>(elements: &[T], directions : Vec<Direction>) -> KKPartition<T>{
    let mut heap: BinaryHeap<KKPartition<T>> = elements
        .into_iter()
        .map(|&x| KKPartition::singleton(x))
        .collect();
    for direction in directions {
        let mut first = heap.pop().expect("heap is empty");
        match heap.pop() {
            None => return first,
            Some(mut snd) => {
                match direction {
                    Direction::Diff => first.merge(snd),
                    Direction::Sum => first.merge_rev(snd),
                    Direction::Fill => {
                        for p in heap {
                            snd.merge_rev(p);
                        }
                        first.merge(snd);
                        return first;
                    },
                }
                heap.push(first);
            }
        }
    }
    panic!("Exhausted directions but heap isn't empty");
}

pub fn ckk<T: Arith>(elements: &[T]) -> KKPartition<T> {
    let mut best_directions = Vec::with_capacity(elements.len());
    let mut directions = Vec::with_capacity(elements.len());
    let heap = elements.iter().map(|x| x.clone()).collect();
    let mut best = elements.iter().map(|x| x.clone()).sum();
    ckk_raw(heap, &mut directions, &mut best, &mut best_directions);
    println!("{:?}", directions);
    reconstruct_ckk(elements, best_directions)
}

fn ckk_raw<T: Arith>(
    mut heap: BinaryHeap<T>,
    directions: &mut Vec<Direction>,
    best: &mut T,
    best_directions: &mut Vec<Direction>,
) {
    let first = heap.pop().expect("heap is empty");
    match heap.pop() {
        None => {
            if *best > first{
                *best = first;
                best_directions.clone_from(directions);
            }
        }
        Some(snd) => {
            let sum_rest: T = heap.iter().map(|x| *x).sum();
            if first >= snd + sum_rest {
                if *best > first - snd - sum_rest {
                    *best = first - snd - sum_rest;
                    best_directions.clone_from(directions);
                    best_directions.push(Direction::Fill);
                }
                return;
            }
            let mut new_heap = heap.clone();
            new_heap.push(first - snd);
            directions.push(Direction::Diff);
            ckk_raw(new_heap, directions, best, best_directions);
            directions.pop();
            directions.push(Direction::Sum);
            heap.push(first + snd);
            ckk_raw(heap, directions, best, best_directions);
            directions.pop();
        }
    }
}
