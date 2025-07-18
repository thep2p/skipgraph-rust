use std::sync::{Arc, Mutex};
use anyhow::Context;
use crate::network::{Message, MessageProcessor, Network};
use crate::network::mock::hub::NetworkHub;

pub struct MockNetwork {
    hub : Arc<NetworkHub>,
    processor: Option<Box<Arc<Mutex<dyn MessageProcessor>>>>,
}

impl MockNetwork {
    pub fn new(hub : Arc<NetworkHub>) -> Self {
        MockNetwork {
            hub: hub.clone(),
            processor: None,
        }
    }
}

impl Network for MockNetwork {
    fn send_message(&self, message: Message) -> anyhow::Result<()> {
        self.hub.route_message(message).context("Failed to route message")?;
        Ok(())
    }

    fn register_processor(&mut self, processor: Box<Arc<Mutex<dyn MessageProcessor>>>) -> anyhow::Result<()> {
        if self.processor.is_some() {
            return Err(anyhow::anyhow!("A message processor is already registered"));
        }
        self.processor = Some(processor);
        Ok(())
    }

    fn start(&mut self) -> anyhow::Result<()> {
        // No-op for mock network
        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        // No-op for mock network
        Ok(())
    }
}