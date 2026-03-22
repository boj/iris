//! `MessageBus` — shared channel-based message passing infrastructure
//! for inter-program communication.
//!
//! Enables Daimon's multi-daemon architecture where cognitive modules
//! communicate asynchronously via named channels. Each channel is a
//! bounded FIFO queue of `Value` messages.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use iris_types::eval::Value;

// ---------------------------------------------------------------------------
// BusError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum BusError {
    /// The named channel does not exist.
    NoSuchChannel(String),
    /// The channel's buffer is at capacity.
    ChannelFull {
        channel: String,
        capacity: usize,
    },
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchChannel(name) => write!(f, "no such channel: {:?}", name),
            Self::ChannelFull { channel, capacity } => {
                write!(f, "channel {:?} full (capacity {})", channel, capacity)
            }
        }
    }
}

impl std::error::Error for BusError {}

// ---------------------------------------------------------------------------
// Channel (internal)
// ---------------------------------------------------------------------------

struct Channel {
    buffer: VecDeque<Value>,
    capacity: usize,
}

impl Channel {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity.min(64)),
            capacity,
        }
    }
}

// ---------------------------------------------------------------------------
// MessageBus
// ---------------------------------------------------------------------------

/// A shared message passing infrastructure for inter-program communication.
///
/// Programs send and receive `Value` messages through named channels.
/// Each channel is a bounded FIFO queue. Channels must be created before
/// use via `create_channel`.
pub struct MessageBus {
    channels: BTreeMap<String, Channel>,
}

impl MessageBus {
    /// Create a new empty message bus with no channels.
    pub fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
        }
    }

    /// Create a named channel with the given buffer capacity.
    ///
    /// If a channel with this name already exists, it is replaced
    /// (existing messages are dropped).
    pub fn create_channel(&mut self, name: &str, capacity: usize) {
        self.channels
            .insert(name.to_string(), Channel::new(capacity));
    }

    /// Send a value to a named channel.
    ///
    /// Returns `Err(BusError::NoSuchChannel)` if the channel does not exist.
    /// Returns `Err(BusError::ChannelFull)` if the channel is at capacity.
    pub fn send(&mut self, channel: &str, value: Value) -> Result<(), BusError> {
        let ch = self
            .channels
            .get_mut(channel)
            .ok_or_else(|| BusError::NoSuchChannel(channel.to_string()))?;

        if ch.buffer.len() >= ch.capacity {
            return Err(BusError::ChannelFull {
                channel: channel.to_string(),
                capacity: ch.capacity,
            });
        }

        ch.buffer.push_back(value);
        Ok(())
    }

    /// Blocking receive from a channel.
    ///
    /// In the current single-threaded orchestrator model, "blocking" means
    /// returning `Ok(None)` when the channel is empty (the caller can retry
    /// on the next cycle). Returns the oldest message if available.
    pub fn recv(&mut self, channel: &str) -> Result<Option<Value>, BusError> {
        let ch = self
            .channels
            .get_mut(channel)
            .ok_or_else(|| BusError::NoSuchChannel(channel.to_string()))?;

        Ok(ch.buffer.pop_front())
    }

    /// Non-blocking receive. Returns `None` if the channel is empty or
    /// does not exist (no error on missing channel for backward compat).
    pub fn try_recv(&mut self, channel: &str) -> Option<Value> {
        self.channels
            .get_mut(channel)
            .and_then(|ch| ch.buffer.pop_front())
    }

    /// Return the number of pending messages in a channel.
    /// Returns 0 if the channel does not exist.
    pub fn pending_count(&self, channel: &str) -> usize {
        self.channels
            .get(channel)
            .map(|ch| ch.buffer.len())
            .unwrap_or(0)
    }

    /// Return the names of all channels.
    pub fn channel_names(&self) -> Vec<&str> {
        self.channels.keys().map(|s| s.as_str()).collect()
    }

    /// Return true if a channel with the given name exists.
    pub fn has_channel(&self, name: &str) -> bool {
        self.channels.contains_key(name)
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_types::eval::Value;

    #[test]
    fn create_and_send_recv() {
        let mut bus = MessageBus::new();
        bus.create_channel("test", 8);

        bus.send("test", Value::Int(42)).unwrap();
        bus.send("test", Value::Int(99)).unwrap();

        assert_eq!(bus.pending_count("test"), 2);
        assert_eq!(bus.recv("test").unwrap(), Some(Value::Int(42)));
        assert_eq!(bus.recv("test").unwrap(), Some(Value::Int(99)));
        assert_eq!(bus.recv("test").unwrap(), None);
    }

    #[test]
    fn try_recv_missing_channel() {
        let mut bus = MessageBus::new();
        assert_eq!(bus.try_recv("nonexistent"), None);
    }

    #[test]
    fn channel_full_error() {
        let mut bus = MessageBus::new();
        bus.create_channel("tiny", 2);

        bus.send("tiny", Value::Int(1)).unwrap();
        bus.send("tiny", Value::Int(2)).unwrap();
        let err = bus.send("tiny", Value::Int(3)).unwrap_err();
        assert_eq!(
            err,
            BusError::ChannelFull {
                channel: "tiny".to_string(),
                capacity: 2,
            }
        );
    }

    #[test]
    fn no_such_channel_error() {
        let mut bus = MessageBus::new();
        let err = bus.send("nope", Value::Unit).unwrap_err();
        assert_eq!(err, BusError::NoSuchChannel("nope".to_string()));
    }

    #[test]
    fn fifo_order() {
        let mut bus = MessageBus::new();
        bus.create_channel("fifo", 16);

        for i in 0..5 {
            bus.send("fifo", Value::Int(i)).unwrap();
        }

        for i in 0..5 {
            assert_eq!(bus.try_recv("fifo"), Some(Value::Int(i)));
        }
        assert_eq!(bus.try_recv("fifo"), None);
    }
}
