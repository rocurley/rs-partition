extern crate cpuprofiler;
extern crate num;
extern crate partition_lib;
extern crate serde_json;
#[macro_use]
extern crate structopt;

use self::cpuprofiler::PROFILER;
use partition_lib::{ckk, gcc, rnp, snp};
use std::io::stdin;
use std::iter::Iterator;
use structopt::StructOpt;

#[derive(StructOpt)]
enum PartitionMethod {
    #[structopt(name = "kk")]
    KK,
    #[structopt(name = "rnp")]
    RNP,
    #[structopt(name = "snp")]
    SNP,
    #[structopt(name = "gcc")]
    GCC,
}

#[derive(StructOpt)]
struct Opt {
    n: u8,
    #[structopt(subcommand)]
    method: PartitionMethod,
}

fn main() {
    let opt = Opt::from_args();
    let elements_result: serde_json::Result<Vec<i32>> = serde_json::from_reader(stdin());
    let elements = elements_result.expect("Couldn't parse input");
    PROFILER.lock().unwrap().start("main.profile").unwrap();
    let partitions = match opt.method {
        PartitionMethod::KK => ckk::n_kk(&elements, opt.n).partitions,
        PartitionMethod::SNP => snp::snp(&elements, opt.n),
        PartitionMethod::GCC => gcc::find_best_partitioning(&elements, opt.n).0,
        PartitionMethod::RNP => {
            if opt.n != 4 {
                panic!("rnp is only implemented for 4 elements right now :(");
            }
            rnp::rnp(&elements).to_vec()
        }
    };
    PROFILER.lock().unwrap().stop().unwrap();
    let output: Vec<Vec<i32>> = partitions
        .iter()
        .map(|subset| subset.to_vec(&elements))
        .collect();
    let output_string = serde_json::to_string_pretty(&output).expect("Serialization failed");
    println!("{}", output_string);
}
