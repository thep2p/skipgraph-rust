use crate::network::Payload::TestMessage;
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
        match message.payload {
            TestMessage(content) => {
                self.seen.insert(content);
                Ok(())
            }
        }
    }
}

/// This test verifies that `MockMessageProcessor` correctly processes and tracks incoming messages routed through a mock network.
#[test]
fn test_mock_message_processor() {
    let hub = NetworkHub::new();
    let identifier = random_identifier();
    let mock_network = NetworkHub::new_mock_network(hub.clone(), identifier).unwrap();
    let processor = MockMessageProcessor::new();
    let message = Message {
        payload: TestMessage("Hello, World!".to_string()),
        target_node_id: identifier,
    };

    assert!(!processor.borrow().has_seen("Hello, World!"));
    assert!(mock_network
        .borrow_mut()
        .register_processor(Box::new(processor.clone()))
        .is_ok());
    assert!(hub.borrow_mut().route_message(message).is_ok());
    assert!(processor.borrow().has_seen("Hello, World!"));
}

/// This test ensures correct routing and processing of messages between mock networks through the `NetworkHub`.
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
        payload: TestMessage("Test message".to_string()),
        target_node_id: id_1,
    };

    assert!(!msg_proc_1.borrow().has_seen("Test message"));
    assert!(mock_net_2.borrow().send_message(message).is_ok());
    assert!(msg_proc_1.borrow().has_seen("Test message"));
}

/// This test sends 10 messages concurrently from mock_net_2 to id_1 and verifies that all messages are processed.
#[test]
fn test_concurrent_message_sending() {
    use crate::network::mock::hub::NetworkHub;
    use std::thread;
    use std::sync::{Arc, Barrier};

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

    // Create 10 different message contents
    let message_contents: Vec<String> = (0..10)
        .map(|i| format!("Concurrent message {}", i))
        .collect();

    // Set up a barrier to synchronize all threads
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    // Spawn 10 threads, each sending a different message
    for i in 0..10 {
        let content = message_contents[i].clone();
        let barrier_clone = barrier.clone();
        let mock_net_2_clone = mock_net_2.clone();
        let id_1_clone = id_1.clone();

        let handle = thread::spawn(move || {
            let message = Message {
                payload: TestMessage(content),
                target_node_id: id_1_clone,
            };

            // Wait for all threads to reach this point
            barrier_clone.wait();

            // Send the message
            mock_net_2_clone.borrow().send_message(message).unwrap();
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify that all messages were received
    let processor = msg_proc_1.borrow();
    for content in message_contents {
        assert!(processor.has_seen(&content), "Message '{}' was not received", content);
    }
}


