pub use base::{Block, Color, Description};

pub mod base;
pub mod binary;
pub mod multicolor;

//pub type Line<B> = smallvec::SmallVec<[B; 32]>;
pub type Line<B> = Box<[B]>;
