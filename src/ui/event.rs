use crossterm::event::{Event as TermEvent, EventStream as TermEventStream, KeyEvent, MouseEvent};
use futures::StreamExt as _;
use std::time::Duration;
use tokio::time::{Interval, MissedTickBehavior};

#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

#[derive(Debug)]
pub struct EventGenerator {
    terminal_es: TermEventStream,
    tick_interval: Interval,
}

impl EventGenerator {
    pub fn new(tick: Duration) -> Self {
        let mut tick_interval = tokio::time::interval(tick);
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        EventGenerator {
            terminal_es: TermEventStream::new(),
            tick_interval,
        }
    }

    pub async fn poll_next(&mut self) -> Event {
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
                _ = self.tick_interval.tick() => {
                    break Event::Tick;
                },
            }
        }
    }
}
