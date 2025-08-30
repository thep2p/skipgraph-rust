use crate::network::{EventProcessorCore, Event};
use anyhow::anyhow;
use std::sync::{Arc, RwLock};
use crate::core::Identifier;

/// A thread-safe wrapper that enforces internal thread-safety for message processors.
/// This type guarantees that all message processing is properly synchronized.
#[derive(Clone)]
pub struct MessageProcessor {
    core: Arc<RwLock<Box<dyn EventProcessorCore>>>,
}

impl MessageProcessor {
    /// Creates a new thread-safe message processor from a core implementation.
    pub fn new(core: Box<dyn EventProcessorCore>) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
        }
    }

    /// Process an incoming message with guaranteed thread-safety.
    pub fn process_incoming_event(&self, origin_id: Identifier, message: Event) -> anyhow::Result<()> {
        let core = self
            .core
            .read()
            .map_err(|_| anyhow!("failed to acquire read lock on message processor"))?;
        core.process_incoming_event(origin_id, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::testutil::fixtures::random_identifier;
    use crate::network::Event;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // A mock implementation of MessageProcessorCore that counts the number of processed messages.
    struct MockMessageProcessorCore {
        counter: Arc<AtomicUsize>,
    }

    impl MockMessageProcessorCore {
        fn new() -> Self {
            Self {
                counter: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn get_counter(&self) -> Arc<AtomicUsize> {
            Arc::clone(&self.counter)
        }
    }

    impl EventProcessorCore for MockMessageProcessorCore {
        fn process_incoming_event(&self, _origin_id: Identifier, _message: Event) -> anyhow::Result<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    /// This test verifies that `MessageProcessor` correctly supports shallow cloning,
    /// i.e., cloned instances share the same underlying core processor.
    #[test]
    fn test_message_processor_shallow_cloning() {
        let mock_core = MockMessageProcessorCore::new();
        let counter_ref = mock_core.get_counter();
        let processor = MessageProcessor::new(Box::new(mock_core));
        let processor_clone = processor.clone();

        let test_message = Event::TestMessage("test".to_string());

        assert_eq!(counter_ref.load(Ordering::SeqCst), 0);

        let origin_id = random_identifier();
        processor.process_incoming_event(origin_id, test_message).unwrap();
        assert_eq!(counter_ref.load(Ordering::SeqCst), 1);

        let origin_id2 = random_identifier();
        let test_message2 = Event::TestMessage("test2".to_string());
        
        processor_clone
            .process_incoming_event(origin_id2, test_message2)
            .unwrap();
        assert_eq!(counter_ref.load(Ordering::SeqCst), 2);
    }
}
