use crossterm::event::{Event as TermEvent, EventStream as TermEventStream, KeyEvent, MouseEvent};
use futures::StreamExt as _;
use std::time::Duration;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    time::{Interval, MissedTickBehavior},
};

#[allow(unused)]
#[derive(Clone, Debug)]
pub enum Event<Msg = ()> {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
    Message(Msg),
}

#[derive(Debug)]
pub struct EventGenerator<Msg = ()> {
    terminal_es: TermEventStream,
    tick_interval: Interval,
    event_receiver: UnboundedReceiver<Msg>,
    event_sender: UnboundedSender<Msg>,
}

impl<Msg> EventGenerator<Msg> {
    pub fn new(tick: Duration) -> Self {
        let mut tick_interval = tokio::time::interval(tick);
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let (event_sender, event_receiver) = unbounded_channel();
        EventGenerator {
            terminal_es: TermEventStream::new(),
            tick_interval,
            event_receiver,
            event_sender,
        }
    }

    pub fn get_app_event_sender(&self) -> UnboundedSender<Msg> {
        self.event_sender.clone()
    }

    pub async fn poll_next(&mut self) -> Event<Msg> {
        loop {
            tokio::select! {
                Some(Ok(evt)) = self.terminal_es.next() => {
                    match evt {
                        TermEvent::Key(key_event) => {
                            break Event::Key(key_event);
                        }
                        TermEvent::Mouse(mouse_event) => {
                            break Event::Mouse(mouse_event);
                        }
                        TermEvent::Resize(col, row) => {
                            break Event::Resize(col, row);
                        }
                        _ => continue
                    }
                },
                Some(message) = self.event_receiver.recv() => {
                    break Event::Message(message);
                },
                _ = self.tick_interval.tick() => {
                    break Event::Tick;
                },
            }
        }
    }
}
