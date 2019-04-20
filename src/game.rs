use crate::{
    display::Display,
    future::*,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Event {
    Idle,
    Quit,
    MouseDown(i32, i32),
    MouseUp(i32, i32),
}

pub trait Game<D>
where
    D: Display,
{
    type Error;

    fn handle<'a>(
        &'a mut self,
        event: &'a Event,
        display: &'a mut D,
    ) -> Fut<'a, Result<bool, Self::Error>>;
}
