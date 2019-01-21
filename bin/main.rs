extern crate cpuprofiler;
extern crate num;
extern crate partition_lib;

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
    /*
    if n_partitions != 4 {
        panic!("Current rnp implementation only works with 4 partitons")
    }
    */
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Couldn't read from stdin");
    let elements_result: Result<Vec<i32>, std::num::ParseIntError> =
        input.trim().split(',').map(|i| i.parse()).collect();
    let elements: Vec<i32> = elements_result.expect("Couldn't parse input");
    PROFILER.lock().unwrap().start("main.profile").unwrap();
    let partitions = snp::snp(&elements, n_partitions);
    PROFILER.lock().unwrap().stop().unwrap();
    println!("{:?}", partitions.to_vec());
}
