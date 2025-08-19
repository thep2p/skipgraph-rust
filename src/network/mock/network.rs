use crate::network::mock::hub::NetworkHub;
use crate::network::{Message, MessageProcessor, Network};
use anyhow::Context;
use std::sync::{Arc, Mutex};

/// MockNetwork is a mock implementation of the Network trait for testing purposes.
/// It does not perform any real network operations but simulates message routing and processing through a `NetworkHub`.
pub struct MockNetwork {
    hub: Arc<Mutex<NetworkHub>>,
    processor: Arc<Option<Box<dyn MessageProcessor>>>,
}

impl MockNetwork {
    /// Creates a new instance of MockNetwork with the given NetworkHub.
    pub fn new(hub: Arc<Mutex<NetworkHub>>) -> Self {
        MockNetwork {
            hub,
            processor: Arc::new(None),
        }
    }

    /// This is the event handler for processing incoming messages come through the mock network.
    /// Arguments:
    /// * `message`: The incoming message to be processed.
    ///   Returns:
    /// * `Result<(), anyhow::Error>`: Returns Ok if the message was processed successfully, or an error if processing failed.
    pub fn incoming_message(&self, message: Message) -> anyhow::Result<()> {
        let processor = self.processor.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No message processor registered"))?;
        
        processor
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire lock on message processor"))?
            .process_incoming_message(message)
            .context("Failed to process incoming message")
    }
}

impl Network for MockNetwork {
    /// Sends a message through the mock network by routing it through the NetworkHub.
    fn send_message(&self, message: Message) -> anyhow::Result<()> {
        self.hub
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire lock on network hub"))?
            .route_message(message)
            .context("Failed to route message")
    }

    /// Registers a message processor to handle incoming messages.
    /// Only one processor can be registered at a time.
    /// If a processor is already registered, an error is returned.
    fn register_processor(
        &mut self,
        processor: Box<Arc<Mutex<dyn MessageProcessor>>>,
    ) -> anyhow::Result<()> {
        match self.processor {
            Some(_) => Err(anyhow::anyhow!("A message processor is already registered")),
            None => {
                self.processor = Some(processor);
                Ok(())
            }
        }
    }
}
