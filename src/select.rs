use structopt::StructOpt;

use arith::Arith;
use brute;
use ckk;
use gcc;
use rnp;
use snp;
use subset::Subset;

#[derive(StructOpt, Debug, Clone, Copy)]
pub enum PartitionMethod {
    #[structopt(name = "kk")]
    KK,
    #[structopt(name = "ckk")]
    CKK,
    #[structopt(name = "rnp")]
    RNP,
    #[structopt(name = "snp")]
    SNP,
    #[structopt(name = "gcc")]
    GCC,
    #[structopt(name = "brute")]
    Brute,
}

pub fn partition_using<T: Arith>(
    method: PartitionMethod,
    elements: &[T],
    n: u8,
) -> Vec<Subset<T, u64>> {
    match method {
        PartitionMethod::KK => ckk::n_kk(elements, n).partitions,
        PartitionMethod::CKK => {
            if n != 2 {
                panic!("ckk is only implemented for 2 partitions right now :(");
            }
            ckk::ckk(elements).to_vec()
        }
        PartitionMethod::SNP => snp::snp(elements, n),
        PartitionMethod::GCC => gcc::find_best_partitioning(elements, n).0,
        PartitionMethod::RNP => {
            if n != 4 {
                panic!("rnp is only implemented for 4 partitions right now :(");
            }
            rnp::rnp(elements).to_vec()
        }
        PartitionMethod::Brute => brute::brute_force(elements, n),
    }
}

#[cfg(test)]
fn pretty_partitioning<T: Arith>(
    partitions: &[Subset<T, u64>],
    elements: &[T],
) -> Vec<(T, Vec<T>)> {
    partitions
        .iter()
        .map(|subset| (subset.sum, subset.to_vec(elements)))
        .collect()
}

#[cfg(test)]
pub fn compare_partitionings(m1: PartitionMethod, m2: PartitionMethod, elements: &[i32], n: u8) {
    let results_1 = partition_using(m1, elements, n);
    let results_2 = partition_using(m2, elements, n);
    let score_1 = results_1.iter().map(|subset| subset.sum).max().unwrap();
    let score_2 = results_2.iter().map(|subset| subset.sum).max().unwrap();
    assert_eq!(
        results_1.len(),
        n as usize,
        "{:?} had wrong number of elements. Expected length was {}, got {:?}",
        m1,
        n,
        pretty_partitioning(&results_1, elements),
    );
    assert_eq!(
        results_2.len(),
        n as usize,
        "{:?} had wrong number of elements. Expected length was {}, got {:?}",
        m2,
        n,
        pretty_partitioning(&results_2, elements),
    );
    assert_eq!(
        score_1,
        score_2,
        "{:?} wins: {:?} got {:?}, {:?} got {:?}",
        if score_1 < score_2 { m1 } else { m2 },
        m1,
        pretty_partitioning(&results_1, elements),
        m2,
        pretty_partitioning(&results_2, elements)
    );
}
