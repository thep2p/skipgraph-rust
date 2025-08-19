use crate::network::mock::hub::NetworkHub;
use crate::network::{Message, MessageProcessor, Network};
use anyhow::Context;
use std::sync::{Arc, Mutex};

/// MockNetwork is a mock implementation of the Network trait for testing purposes.
/// It does not perform any real network operations but simulates message routing and processing through a `NetworkHub`.
pub struct MockNetwork {
    hub: Arc<Mutex<NetworkHub>>,
    processor: Arc<Mutex<Option<Box<dyn MessageProcessor>>>>,
}

impl MockNetwork {
    /// Creates a new instance of MockNetwork with the given NetworkHub.
    pub fn new(hub: Arc<Mutex<NetworkHub>>) -> Self {
        MockNetwork {
            hub,
            processor: Arc::new(Mutex::new(None)),
        }
    }

    /// This is the event handler for processing incoming messages come through the mock network.
    /// Arguments:
    /// * `message`: The incoming message to be processed.
    ///   Returns:
    /// * `Result<(), anyhow::Error>`: Returns Ok if the message was processed successfully, or an error if processing failed.
    pub fn incoming_message(&self, message: Message) -> anyhow::Result<()> {
        let mut processor_guard = self.processor
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire lock on processor container"))?;
        
        let processor = processor_guard.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No message processor registered"))?;
        
        processor
            .process_incoming_message(message)
            .context("Failed to process incoming message")
    }
}

impl Clone for MockNetwork {
    fn clone(&self) -> Self {
        MockNetwork {
            hub: Arc::clone(&self.hub),
            processor: Arc::clone(&self.processor),
        }
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
        processor: Box<dyn MessageProcessor>,
    ) -> anyhow::Result<()> {
        let mut processor_guard = self.processor
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire lock on processor container"))?;
        
        match processor_guard.as_ref() {
            Some(_) => Err(anyhow::anyhow!("A message processor is already registered")),
            None => {
                *processor_guard = Some(processor);
                Ok(())
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Network> {
        Box::new(self.clone())   
    }
}
