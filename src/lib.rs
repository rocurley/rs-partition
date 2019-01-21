#![feature(nll)]
#![feature(test)]
#![feature(range_contains)]
#![allow(unknown_lints)]
#![warn(clippy::all, clippy::module_name_repetitions, clippy::unseparated_literal_suffix)]

extern crate num;
#[cfg(test)]
#[macro_use]
extern crate proptest;
mod arith;
pub mod ckk;
pub mod rnp;
pub mod ess;
pub mod gcc;
pub mod snp;
pub mod subset;
