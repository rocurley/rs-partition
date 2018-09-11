extern crate cpuprofiler;
extern crate num;

use cpuprofiler::PROFILER;
use num::Integer;
use std::convert::From;
//use std::env::args;
use std::io::stdin;
use std::iter::{Iterator, Sum};
use std::ops::{AddAssign, SubAssign};
use std::fmt::{Debug, Display};

trait Arith: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display{}
impl<T> Arith for T where T: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display{}

mod rnp;

fn main() {
    //let string_args: Vec<String> = args().collect();
    //let n_partitions : u8 = string_args[1].parse().expect("Couldn't parse argument");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Couldn't read from stdin");
    let elements_result: Result<Vec<f64>, std::num::ParseFloatError> =
        input.trim().split(",").map(|i| i.parse()).collect();
    let elements: Vec<i32> = elements_result
        .expect("Couldn't parse input")
        .into_iter()
        .map(|n| (n * 1000000.) as i32)
        .collect();
    println!("Total weight: {}", elements.iter().sum::<i32>());
    //elements.sort_by_key(|x| -x);
    PROFILER
        .lock()
        .unwrap()
        .start("./my-prof.profile")
        .expect("Couldn't start");
    let partition = rnp::ckk(&elements);
    PROFILER.lock().unwrap().stop().expect("Couldn't stop");
    println!("{:?}", partition);
}
