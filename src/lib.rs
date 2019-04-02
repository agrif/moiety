#![feature(futures_api)]
#![feature(async_await)]
#![feature(await_macro)]
#![feature(arbitrary_self_types)]

#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;

#[macro_use]
mod future;
pub use future::*;
mod filesystem;
pub use filesystem::*;
mod resources;
pub use resources::*;
mod map;
pub use map::*;
mod format;
pub use format::*;
mod mhk;
pub use mhk::*;
mod direct;
pub use direct::*;
mod json;
pub use json::*;
