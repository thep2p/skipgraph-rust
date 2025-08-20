use std::sync::{Arc, RwLock};
use crate::network::{Message, MessageProcessorCore};

/// A thread-safe wrapper that enforces internal thread-safety for message processors.
/// This type guarantees that all message processing is properly synchronized.
#[derive(Clone)]
pub struct MessageProcessor {
    core: Arc<RwLock<Box<dyn MessageProcessorCore>>>,
}

impl MessageProcessor {
    /// Creates a new thread-safe message processor from a core implementation.
    pub fn new(core: Box<dyn MessageProcessorCore>) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
        }
    }

    /// Process an incoming message with guaranteed thread-safety.
    pub fn process_incoming_message(&self, message: Message) -> anyhow::Result<()> {
        let core = self.core.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on message processor"))?;
        core.process_incoming_message(message)
    }
}