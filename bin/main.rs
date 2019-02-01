extern crate cpuprofiler;
extern crate num;
extern crate partition_lib;
extern crate serde_json;
extern crate structopt;

use self::cpuprofiler::PROFILER;
use partition_lib::select;
use std::io::stdin;
use std::iter::Iterator;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    n: u8,
    #[structopt(subcommand)]
    method: select::PartitionMethod,
}

fn main() {
    let opt = Opt::from_args();
    let elements_result: serde_json::Result<Vec<i32>> = serde_json::from_reader(stdin());
    let elements = elements_result.expect("Couldn't parse input");
    PROFILER.lock().unwrap().start("main.profile").unwrap();
    let partitions = select::partition_using(opt.method, &elements, opt.n);
    PROFILER.lock().unwrap().stop().unwrap();
    let output: Vec<Vec<i32>> = partitions
        .iter()
        .map(|subset| subset.to_vec(&elements))
        .collect();
    let output_string = serde_json::to_string_pretty(&output).expect("Serialization failed");
    println!("{}", output_string);
}
