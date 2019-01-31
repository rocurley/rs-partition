#![feature(nll)]
#![feature(test)]
#![feature(range_contains)]
#![feature(range_is_empty)]
#![allow(unknown_lints)]
#![warn(
    clippy::all,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix
)]

extern crate num;
#[cfg(test)]
#[macro_use]
extern crate proptest;
extern crate itertools;
mod arith;
#[cfg(test)]
mod benchmark_data;
pub mod ckk;
pub mod ess;
pub mod gcc;
pub mod rnp;
pub mod snp;
pub mod subset;
