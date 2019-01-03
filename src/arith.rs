use num::Integer;
use std::convert::From;
use std::fmt::{Debug, Display};
use std::iter::Sum;
use std::ops::{AddAssign, SubAssign};

pub trait Arith:
    Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display
{
}
impl<T> Arith for T where
    T: Integer + AddAssign + SubAssign + From<u8> + Clone + Copy + Sum + Debug + Display
{}
