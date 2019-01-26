extern crate cpuprofiler;
extern crate num;
extern crate partition_lib;
extern crate serde_json;

use self::cpuprofiler::PROFILER;
use num::Integer;
use partition_lib::snp;
use std::convert::From;
use std::env::args;
use std::fmt::{Debug, Display};
use std::io::stdin;
use std::iter::{Iterator, Sum};
use std::ops::{AddAssign, SubAssign};

trait Arith: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display {}
impl<T> Arith for T where
    T: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display
{
}

fn main() {
    let string_args: Vec<String> = args().collect();
    let n_partitions: u8 = string_args[1].parse().expect("Couldn't parse argument");
    let elements_result: serde_json::Result<Vec<i32>> = serde_json::from_reader(stdin());
    let elements = elements_result.expect("Couldn't parse input");
    PROFILER.lock().unwrap().start("main.profile").unwrap();
    let partitions = snp::snp(&elements, n_partitions);
    PROFILER.lock().unwrap().stop().unwrap();
    println!("{:?}", partitions.to_vec());
}
