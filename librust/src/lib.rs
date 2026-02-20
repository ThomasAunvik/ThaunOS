#![no_std]
#![feature(c_variadic)]

mod stdlib;
mod stdio;
mod string;

pub use stdlib::*;
pub use stdlib::abort::*;

pub use stdio::*;

pub use string::*;
pub use string::memmove::*;
pub use string::strlen::*;