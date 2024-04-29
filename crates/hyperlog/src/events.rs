use std::sync::{Arc, Mutex};

use bus::{Bus, BusReader};

use crate::commander::Command;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Event {
    CommandEvent { command: Command },
}

#[derive(Clone)]
pub struct Events {
    bus: Arc<Mutex<Bus<Event>>>,
}

impl Default for Events {
    fn default() -> Self {
        let bus = Bus::new(10);

        Self {
            bus: Arc::new(Mutex::new(bus)),
        }
    }
}

impl Events {
    pub fn enque_command(&self, cmd: Command) -> anyhow::Result<()> {
        self.bus
            .lock()
            .unwrap()
            .broadcast(Event::CommandEvent { command: cmd });

        Ok(())
    }

    pub fn subscribe(&self) -> anyhow::Result<BusReader<Event>> {
        let rx = self.bus.lock().unwrap().add_rx();

        Ok(rx)
    }
}

#[cfg(test)]
mod test {
    use similar_asserts::assert_eq;

    use crate::{commander::Command, events::Event};

    use super::Events;

    #[test]
    fn can_enque_command() -> anyhow::Result<()> {
        let events = Events::default();

        events.enque_command(Command::CreateRoot {
            root: "some-root".into(),
        })?;

        Ok(())
    }

    #[test]
    fn can_deque_command() -> anyhow::Result<()> {
        let events = Events::default();
        let mut rx = events.subscribe()?;

        std::thread::spawn(move || {
            events
                .enque_command(Command::CreateRoot {
                    root: "some-root".into(),
                })
                .unwrap();
        });

        let event = rx.recv()?;

        assert_eq!(
            Event::CommandEvent {
                command: Command::CreateRoot {
                    root: "some-root".into()
                },
            },
            event
        );

        Ok(())
    }

    #[test]
    fn is_broadcast() -> anyhow::Result<()> {
        let events = Events::default();
        let mut rx1 = events.subscribe()?;
        let mut rx2 = events.subscribe()?;

        std::thread::spawn(move || {
            events
                .enque_command(Command::CreateRoot {
                    root: "some-root".into(),
                })
                .unwrap();

            events
                .enque_command(Command::CreateRoot {
                    root: "another-event".into(),
                })
                .unwrap();
        });

        let event = rx1.recv()?;
        let same_event = rx2.recv()?;

        assert_eq!(event, same_event);

        assert_eq!(
            Event::CommandEvent {
                command: Command::CreateRoot {
                    root: "some-root".into()
                },
            },
            event
        );

        Ok(())
    }
}
