use crate::network::mock::hub::NetworkHub;
use crate::network::{Message, MessageProcessor, Network};
use anyhow::Context;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MockNetwork {
    hub: Rc<RefCell<NetworkHub>>,
    processor: Option<Box<Rc<RefCell<dyn MessageProcessor>>>>,
}

impl MockNetwork {
    pub fn new(hub: Rc<RefCell<NetworkHub>>) -> Self {
        MockNetwork {
            hub,
            processor: None,
        }
    }
}

impl Network for MockNetwork {
    fn send_message(&self, message: Message) -> anyhow::Result<()> {
        self.hub
            .borrow()
            .route_message(message)
            .context("Failed to route message")?;
        Ok(())
    }

    fn register_processor(
        &mut self,
        processor: Box<Rc<RefCell<dyn MessageProcessor>>>,
    ) -> anyhow::Result<()> {
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
