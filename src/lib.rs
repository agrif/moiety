#![feature(futures_api)]
#![feature(async_await)]
#![feature(await_macro)]
#![feature(arbitrary_self_types)]
#![feature(slice_patterns)]
#![feature(copy_within)]

#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod future;

pub mod filesystem;

#[macro_use]
mod resource_type;
pub use resource_type::*;

#[macro_use]
mod stack;
pub use stack::*;

mod resources;
pub use resources::*;
mod map;
pub use map::*;
mod format;
pub use format::*;
mod direct;
pub use direct::*;
mod json;
pub use json::*;
mod yaml;
pub use yaml::*;
mod png;
pub use self::png::*;
pub mod mhk;
pub use mhk::{
    MhkError,
    MhkFormat,
    MhkMap,
};

pub mod riven;
