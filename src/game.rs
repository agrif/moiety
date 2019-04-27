use crate::{
    display::Display,
    future::*,
};
use futures::{
    sink::SinkExt,
    stream::StreamExt,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Event {
    Idle,
    Quit,
    MouseDown(i32, i32),
    MouseUp(i32, i32),
}

#[derive(Debug)]
pub struct EventPump {
    sender: EventSender,
    receiver: futures::channel::mpsc::Receiver<Event>,
}

#[derive(Debug, Clone)]
pub struct EventSender {
    sender: futures::channel::mpsc::Sender<Event>,
}

impl EventPump {
    pub fn new() -> Self {
        let (sender, receiver) = futures::channel::mpsc::channel(0);
        EventPump {
            sender: EventSender { sender },
            receiver,
        }
    }

    pub fn sender(&self) -> EventSender { self.sender.clone() }

    pub async fn pump(&mut self) -> Event {
        // clear events that happened while we weren't pumping...
        while self.receiver.try_next().is_ok() {}
        if let Some(ev) = await!(self.receiver.next()) {
            ev
        } else {
            Event::Quit
        }
    }
}

impl EventSender {
    pub async fn send(&mut self, event: Event) {
        // try to send but if it fails, oh well
        let _ = await!(self.sender.send(event));
    }
}

pub trait Game<'a, D>
where
    D: Display,
{
    type Error: Send + 'static;

    fn start(
        self,
        pump: EventPump,
        display: D,
    ) -> Fut<'a, Result<(), Self::Error>>;
}
