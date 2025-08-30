use crate::network::mock::hub::NetworkHub;
use crate::network::{Event, MessageProcessor, Network};
use anyhow::{anyhow, Context};
use std::sync::RwLock;
use crate::core::Identifier;

/// MockNetwork is a mock implementation of the Network trait for testing purposes.
/// It does not perform any real network operations but simulates message routing and processing through a `NetworkHub`.
/// 
/// Thread-safety is handled internally using Mutex for the processor, following a Go-like approach
/// where the struct can be safely shared via Arc<MockNetwork> without external locking.
/// MessageProcessor is inherently thread-safe, so we only need a simple Option wrapper.
pub struct MockNetwork {
    core: RwLock<InnerMockNetwork>,
}

struct InnerMockNetwork {
    hub: NetworkHub,
    processor: Option<MessageProcessor>,
    id: Identifier, // Identifier of the mock network
}

impl MockNetwork {
    /// Creates a new instance of MockNetwork with the given NetworkHub.
    pub fn new(id: Identifier, hub: NetworkHub) -> Self {
        MockNetwork {
            core: RwLock::new(InnerMockNetwork {
                hub,
                processor: None,
                id,
            }),
        }
    }

    /// This is the event handler for processing incoming messages come through the mock network.
    /// Arguments:
    /// * `message`: The incoming message to be processed.
    ///   Returns:
    /// * `Result<(), anyhow::Error>`: Returns Ok if the message was processed successfully, or an error if processing failed.
    pub fn incoming_message(&self, origin_id: Identifier, message: Event) -> anyhow::Result<()> {
        let core_guard = self.core
            .read()
            .map_err(|_| anyhow!("failed to acquire read lock on core"))?;
        
        let processor = match core_guard.processor.as_ref() {
            Some(p) => p,
            None => return Err(anyhow!("no message processor registered")),
        };
        
        processor
            .process_incoming_event(origin_id, message)
            .context("failed to process incoming message")
    }
}

impl Clone for MockNetwork {
    fn clone(&self) -> Self {
        let core_guard = self.core.read().unwrap();
        MockNetwork {
            core: RwLock::new(InnerMockNetwork {
                hub: core_guard.hub.clone(),
                processor: core_guard.processor.clone(), // Share processor state between clones
                id: core_guard.id,
            }),
        }
    }
}

impl Network for MockNetwork {
    /// Sends a message through the mock network by routing it through the NetworkHub.
    fn send_event(&self, target_id: Identifier, message: Event) -> anyhow::Result<()> {
        let core_guard = self.core
            .read()
            .map_err(|_| anyhow!("failed to acquire read lock on core"))?;
        
        core_guard.hub
            .route_message(core_guard.id, target_id, message)
            .context("failed to route message")
    }

    /// Registers a message processor to handle incoming messages.
    /// Only one processor can be registered at a time.
    /// If a processor is already registered, an error is returned.
    fn register_processor(
        &self,
        processor: MessageProcessor,
    ) -> anyhow::Result<()> {
        let mut core_guard = self.core
            .write()
            .map_err(|_| anyhow!("failed to acquire write lock on core"))?;
        
        match core_guard.processor.as_ref() {
            Some(_) => Err(anyhow!("a message processor is already registered")),
            None => {
                core_guard.processor = Some(processor);
                Ok(())
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Network> {
        Box::new(self.clone())
    }
}
