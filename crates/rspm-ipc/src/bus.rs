//! In-process pub/sub bus used by the daemon.

use tokio::sync::broadcast;

use rspm_protocol::Event;

/// Broadcast bus for daemon events.
#[derive(Clone, Debug)]
pub struct PubSubBus {
    sender: broadcast::Sender<Event>,
}

impl PubSubBus {
    /// Creates a new bus with the provided channel capacity.
    ///
    /// ```
    /// let bus = rspm_ipc::PubSubBus::new(16);
    /// let _rx = bus.subscribe();
    /// ```
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribes to future events.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Publishes an event to all current subscribers.
    pub fn publish(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}
