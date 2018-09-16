extern crate cpuprofiler;
extern crate num;
extern crate partition_lib;

use cpuprofiler::PROFILER;
use num::Integer;
use std::convert::From;
use std::env::args;
use std::fs::File;
use std::io::{Read, stdin};
use std::iter::{Iterator, Sum};
use std::ops::{AddAssign, SubAssign};
use std::fmt::{Debug, Display};
use partition_lib::{ckk};

trait Arith: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display{}
impl<T> Arith for T where T: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display{}

fn main() {
    /*
    let string_args: Vec<String> = args().collect();
    let n_partitions : usize = string_args[1].parse().expect("Couldn't parse argument");
    let mut input = String::new();
    let mut f = File::open("rates").expect("rates not found");
    f
        .read_to_string(&mut input)
        .expect("Couldn't read from stdin");
    let elements_result: Result<Vec<i32>, std::num::ParseIntError> =
        input.trim().split(",").map(|i| i.parse()).collect();
    let elements: Vec<i32> = elements_result.expect("Couldn't parse input");
    println!("Total weight: {}", elements.iter().sum::<i32>());
    //elements.sort_by_key(|x| -x);
    PROFILER
        .lock()
        .unwrap()
        .start("./my-prof.profile")
        .expect("Couldn't start");
    ckk::ckk(&elements);
    let partition = ckk::n_kk_score(&elements, 4);
    PROFILER.lock().unwrap().stop().expect("Couldn't stop");
    println!("{:?}", partition);
    */
    let elements = vec![
        403188, 4114168, 4114168, 5759835, 5759835, 5759835, 2879917, 8228336, 8228336,
        8228336, 8228336, 8228336, 8228336, 8228336, 2057084, 2057084, 2057084, 2057084,
        2057084, 2057084, 2057084, 9599726, 9599726, 9599726, 9599726, 9599726, 9599726,
        537584, 537584, 537584,
    ];
    PROFILER
        .lock()
        .unwrap()
        .start("./rnp.profile")
        .expect("Couldn't start");
    println!("{:?}", ckk::rnp(&elements));
    PROFILER.lock().unwrap().stop().expect("Couldn't stop");
}
