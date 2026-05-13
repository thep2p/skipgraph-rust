use crate::core::Identifier;
use crate::network::{Event, MessageProcessor, Network};

/// NoopNetwork is a mock implementation of the Network trait that does not perform any operations.
/// It is used for testing scenarios where a Network is required but its behaviour is irrelevant.
#[allow(dead_code)]
pub struct NoopNetwork;

impl Network for NoopNetwork {
    fn send_event(&self, _target_id: Identifier, _event: Event) -> anyhow::Result<()> {
        Ok(())
    }

    fn register_processor(&self, _processor: MessageProcessor) -> anyhow::Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Network> {
        Box::new(NoopNetwork)
    }
}

/// Creates a new boxed NoopNetwork suitable for tests that need to satisfy the
/// `Network` trait without exercising real routing.
#[allow(dead_code)]
pub fn new_noop_network() -> Box<dyn Network> {
    Box::new(NoopNetwork)
}
