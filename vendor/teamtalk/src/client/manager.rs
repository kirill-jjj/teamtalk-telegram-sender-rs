use crate::events::Event;
use crate::types::{ClientId, CommandId};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone)]
pub struct ClientEvent {
    pub client_id: ClientId,
    pub label: Option<String>,
    pub event: Event,
    pub command_id: Option<CommandId>,
    pub at: SystemTime,
}

#[derive(Debug, Clone, Default)]
pub struct ClientHealth {
    pub last_event: Option<Event>,
    pub last_event_at: Option<SystemTime>,
    pub last_poll_at: Option<SystemTime>,
}

pub struct ClientManager {
    clients: Vec<crate::client::Client>,
    health: HashMap<ClientId, ClientHealth>,
    queue: VecDeque<usize>,
    poll_timeout_ms: i32,
    tick_sleep: Duration,
    tx: Sender<ClientEvent>,
    rx: Receiver<ClientEvent>,
}

impl Default for ClientManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            clients: Vec::new(),
            health: std::collections::HashMap::new(),
            queue: VecDeque::new(),
            poll_timeout_ms: 0,
            tick_sleep: Duration::from_millis(1),
            tx,
            rx,
        }
    }

    pub fn add_client(&mut self, client: crate::client::Client) {
        let id = client.id();
        self.health.insert(id, ClientHealth::default());
        self.queue.push_back(self.clients.len());
        self.clients.push(client);
    }

    pub fn remove_client(&mut self, id: ClientId) {
        if let Some(pos) = self.clients.iter().position(|c| c.id() == id) {
            self.clients.remove(pos);
            self.health.remove(&id);
            self.queue = self
                .queue
                .iter()
                .filter_map(|&idx| {
                    if idx == pos {
                        None
                    } else if idx > pos {
                        Some(idx - 1)
                    } else {
                        Some(idx)
                    }
                })
                .collect();
        }
    }

    pub fn set_poll_timeout(&mut self, timeout_ms: i32) {
        self.poll_timeout_ms = timeout_ms;
    }

    pub fn set_tick_sleep(&mut self, sleep: Duration) {
        self.tick_sleep = sleep;
    }

    pub fn events(&self) -> &Receiver<ClientEvent> {
        &self.rx
    }

    pub fn health_snapshot(&self, id: ClientId) -> Option<ClientHealth> {
        self.health.get(&id).cloned()
    }

    pub fn run_once(&mut self) {
        let now = SystemTime::now();
        let mut processed = 0;
        let queue_len = self.queue.len();
        for _ in 0..queue_len {
            if let Some(idx) = self.queue.pop_front() {
                if let Some(client) = self.clients.get(idx) {
                    if let Some((event, msg)) = client.poll(self.poll_timeout_ms) {
                        let command_id = match event {
                            Event::CmdProcessing | Event::CmdError | Event::CmdSuccess => {
                                Some(CommandId(msg.source()))
                            }
                            _ => None,
                        };
                        let evt = ClientEvent {
                            client_id: client.id(),
                            label: client.label(),
                            event,
                            command_id,
                            at: now,
                        };
                        let _ = self.tx.send(evt.clone());
                        let entry = self.health.entry(client.id()).or_default();
                        entry.last_event = Some(event);
                        entry.last_event_at = Some(now);
                    }
                    let entry = self.health.entry(client.id()).or_default();
                    entry.last_poll_at = Some(now);
                }
                self.queue.push_back(idx);
                processed += 1;
            }
        }
        if processed == 0 {
            std::thread::sleep(self.tick_sleep);
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            let start = Instant::now();
            self.run_once();
            if self.tick_sleep > Duration::ZERO {
                let elapsed = start.elapsed();
                if elapsed < self.tick_sleep {
                    std::thread::sleep(self.tick_sleep - elapsed);
                }
            }
        }
    }

    pub fn wait_cmd(
        &mut self,
        client_id: ClientId,
        cmd_id: CommandId,
        timeout_ms: i32,
    ) -> Result<Event, WaitError> {
        let deadline = if timeout_ms < 0 {
            None
        } else {
            Some(Instant::now() + Duration::from_millis(timeout_ms as u64))
        };
        loop {
            self.run_once();
            while let Ok(evt) = self.rx.try_recv() {
                if evt.client_id == client_id
                    && evt.command_id == Some(cmd_id)
                    && matches!(evt.event, Event::CmdSuccess | Event::CmdError)
                {
                    return match evt.event {
                        Event::CmdSuccess => Ok(evt.event),
                        Event::CmdError => Err(WaitError::CommandFailed),
                        _ => unreachable!(),
                    };
                }
            }
            if let Some(deadline) = deadline
                && Instant::now() >= deadline
            {
                return Err(WaitError::Timeout);
            }
        }
    }

    pub fn wait_cmd_ok(
        &mut self,
        client_id: ClientId,
        cmd_id: CommandId,
        timeout_ms: i32,
    ) -> Result<(), WaitError> {
        self.wait_cmd(client_id, cmd_id, timeout_ms).map(|_| ())
    }

    pub fn wait_cmd_any(
        &mut self,
        cmd_id: CommandId,
        timeout_ms: i32,
    ) -> Result<ClientId, WaitError> {
        let deadline = if timeout_ms < 0 {
            None
        } else {
            Some(Instant::now() + Duration::from_millis(timeout_ms as u64))
        };
        loop {
            self.run_once();
            while let Ok(evt) = self.rx.try_recv() {
                if evt.command_id == Some(cmd_id)
                    && matches!(evt.event, Event::CmdSuccess | Event::CmdError)
                {
                    return match evt.event {
                        Event::CmdSuccess => Ok(evt.client_id),
                        Event::CmdError => Err(WaitError::CommandFailed),
                        _ => unreachable!(),
                    };
                }
            }
            if let Some(deadline) = deadline
                && Instant::now() >= deadline
            {
                return Err(WaitError::Timeout);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitError {
    Timeout,
    CommandFailed,
}

#[cfg(test)]
mod tests {
    use super::{ClientEvent, ClientManager, WaitError};
    use crate::events::Event;
    use crate::types::{ClientId, CommandId};
    use std::time::{Duration, SystemTime};

    #[test]
    fn wait_cmd_ok_returns_success() {
        let mut manager = ClientManager::new();
        manager.set_tick_sleep(Duration::ZERO);
        let client_id = ClientId(1);
        let cmd_id = CommandId(42);
        let evt = ClientEvent {
            client_id,
            label: None,
            event: Event::CmdSuccess,
            command_id: Some(cmd_id),
            at: SystemTime::now(),
        };
        manager.tx.send(evt).expect("send");
        assert!(manager.wait_cmd_ok(client_id, cmd_id, 0).is_ok());
    }

    #[test]
    fn wait_cmd_any_returns_client() {
        let mut manager = ClientManager::new();
        manager.set_tick_sleep(Duration::ZERO);
        let cmd_id = CommandId(7);
        let evt = ClientEvent {
            client_id: ClientId(2),
            label: None,
            event: Event::CmdSuccess,
            command_id: Some(cmd_id),
            at: SystemTime::now(),
        };
        manager.tx.send(evt).expect("send");
        let got = manager.wait_cmd_any(cmd_id, 0).expect("ok");
        assert_eq!(got, ClientId(2));
    }

    #[test]
    fn wait_cmd_any_times_out() {
        let mut manager = ClientManager::new();
        manager.set_tick_sleep(Duration::ZERO);
        let cmd_id = CommandId(99);
        let err = manager.wait_cmd_any(cmd_id, 0).expect_err("timeout");
        assert_eq!(err, WaitError::Timeout);
    }
}
