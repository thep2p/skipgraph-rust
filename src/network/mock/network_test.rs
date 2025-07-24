use crate::network::MessageType::TestMessage;
use crate::network::{Message, MessageProcessor, Network};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::core::testutil::fixtures::random_identifier;
use crate::network::mock::hub::NetworkHub;

struct MockMessageProcessor {
    seen: HashSet<String>,
}

impl MockMessageProcessor {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(MockMessageProcessor {
            seen: HashSet::new(),
        }))
    }

    fn has_seen(&self, content: &str) -> bool {
        self.seen.contains(content)
    }
}

impl MessageProcessor for MockMessageProcessor {
    fn process_incoming_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message.message_type {
            TestMessage(content) => {
                self.seen.insert(content);
                Ok(())
            }
            _ => Err(anyhow::anyhow!(format!(
                "Unknown message type {:?}",
                message.message_type
            ))),
        }
    }
}

/// Test for the MockMessageProcessor to ensure it correctly processes messages.
#[test]
fn test_mock_message_processor() {
    let hub = NetworkHub::new();
    let identifier = random_identifier();
    let mock_network = NetworkHub::new_mock_network(hub, identifier).unwrap();
    let mut processor = MockMessageProcessor::new();
    let message = Message {
        message_type: TestMessage("Hello, World!".to_string()),
        target_node_id: random_identifier(),
        payload: Box::new(()),
    };

    assert!(!processor.borrow().has_seen("Hello, World!"));
    processor
        .borrow_mut()
        .process_incoming_message(message)
        .unwrap();
    assert!(processor.borrow().has_seen("Hello, World!"));
}

#[test]
fn test_hub_route_message() {
    use crate::network::mock::hub::NetworkHub;

    let hub = NetworkHub::new();

    let id_1 = random_identifier();
    let mock_net_1 = NetworkHub::new_mock_network(hub.clone(), id_1).unwrap();
    let msg_proc_1 = MockMessageProcessor::new();
    mock_net_1
        .borrow_mut()
        .register_processor(Box::new(msg_proc_1.clone()))
        .expect("Failed to register message processor");

    let id_2 = random_identifier();
    let mock_net_2 = NetworkHub::new_mock_network(hub, id_2).unwrap();

    let message = Message {
        message_type: TestMessage("Test message".to_string()),
        target_node_id: id_1,
        payload: Box::new(()),
    };

    assert!(!msg_proc_1.borrow().has_seen("Test message"));
    assert!(mock_net_2.borrow().send_message(message).is_ok());
    assert!(msg_proc_1.borrow().has_seen("Test message"));
}
