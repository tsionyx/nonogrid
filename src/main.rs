mod board;
mod reader;
mod render;
mod utils;

#[macro_use]
extern crate serde_derive;

use render::{Renderer, ShellRenderer};
use std::rc::Rc;

fn main() {
    let b = reader::MyFormat::read_board("examples/hello.toml");
    println!("{}", ShellRenderer { board: Rc::new(b) }.render());
}
