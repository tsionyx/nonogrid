use super::super::utils;

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::Sized;
use std::ops::{Add, Sub};

pub trait Color
where
    Self: Debug
        + PartialEq
        + Eq
        + Hash
        + Copy
        + Clone
        + PartialOrd
        + Ord
        + Add<Output = Self>
        + Sub<Output = Result<Self, String>>,
{
    fn initial() -> Self;
    fn blank() -> Self;
    fn is_solved(&self) -> bool;
    fn solution_rate(&self) -> f64;
    fn is_updated_with(&self, new: &Self) -> Result<bool, String>;
    fn variants(&self) -> HashSet<Self>
    where
        Self: Sized;
}

pub trait Block
where
    Self: Debug + PartialEq + Eq + Hash + Default + Clone,
{
    type Color: Color;

    fn from_str(s: &str) -> Self;
    fn partial_sums(desc: &[Self]) -> Vec<usize>
    where
        Self: Sized;

    fn size(&self) -> usize;
    fn color(&self) -> Self::Color;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Description<T: Block>
where
    T: Block,
{
    pub vec: Vec<T>,
}

impl<T> Description<T>
where
    T: Block,
{
    pub fn new(mut vec: Vec<T>) -> Description<T> {
        // remove zero blocks
        utils::remove(&mut vec, T::default());
        Description { vec }
    }
}
