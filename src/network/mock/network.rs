use crate::network::{Message, MessageProcessor, Network};
use anyhow::Context;
use std::sync::{Arc, Mutex};
use crate::network::mock::hub::NetworkHub;

/// MockNetwork is a mock implementation of the Network trait for testing purposes.
/// It does not perform any real network operations but simulates message routing and processing through a `NetworkHub`.
#[derive(Debug)]
pub struct MockNetwork {
    hub: Arc<Mutex<NetworkHub>>,
    node_id: crate::core::Identifier,
    processor: Option<Box<Arc<Mutex<dyn MessageProcessor>>>>,
}

impl MockNetwork {
    /// Creates a new instance of MockNetwork with the given NetworkHub and node identifier.
    pub fn new(hub: Arc<Mutex<NetworkHub>>, node_id: crate::core::Identifier) -> Self {
        MockNetwork {
            hub,
            node_id,
            processor: None,
        }
    }

    /// This is the event handler for processing incoming messages come through the mock network.
    /// Arguments:
    /// * `message`: The incoming message to be processed.
    /// * `origin_id`: The identifier of the node that sent the message.
    ///   Returns:
    /// * `Result<(), anyhow::Error>`: Returns Ok if the message was processed successfully, or an error if processing failed.
    pub fn incoming_message(
        &self,
        message: Message,
        origin_id: crate::core::Identifier,
    ) -> anyhow::Result<()> {
        if let Some(ref processor) = self.processor {
            processor
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire lock on message processor"))?
                .process_incoming_message(origin_id, message)
                .context("Failed to process incoming message")?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No message processor registered"))
        }
    }
}

impl Network for MockNetwork {
    /// Sends a message through the mock network by routing it through the NetworkHub.
    fn send_message(&self, message: Message) -> anyhow::Result<()> {
        self.hub
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire lock on network hub"))?
            .route_message(message, self.node_id)
            .context("Failed to route message")?;
        Ok(())
    }

    /// Registers a message processor to handle incoming messages.
    /// Only one processor can be registered at a time.
    /// If a processor is already registered, an error is returned.
    fn register_processor(
        &mut self,
        processor: Box<Arc<Mutex<dyn MessageProcessor>>>,
    ) -> anyhow::Result<()> {
        if self.processor.is_some() {
            return Err(anyhow::anyhow!("A message processor is already registered"));
        }
        self.processor = Some(processor);
        Ok(())
    }
}
