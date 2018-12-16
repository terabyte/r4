use std::sync::Arc;

pub mod process;
pub mod stdout;

pub type Line = Arc<str>;

pub trait Stream {
    fn write_line(&mut self, Line);
    fn rclosed(&mut self) -> bool;
    fn close(&mut self);
}