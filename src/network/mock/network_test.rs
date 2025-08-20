use crate::core::testutil::fixtures::random_identifier;
use crate::network::mock::hub::NetworkHub;
use crate::network::Payload::TestMessage;
use crate::network::{Message, MessageProcessor, Network};
use std::collections::HashSet;
use std::sync::{Arc, Barrier, Mutex, RwLock};
use std::thread;

struct MockMessageProcessor {
    inner: Arc<RwLock<MockMessageProcessorInner>>,
}

struct MockMessageProcessorInner {
    seen: HashSet<String>,
}

impl MockMessageProcessor {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(MockMessageProcessor {
            inner: Arc::new(RwLock::new(MockMessageProcessorInner {
                seen: HashSet::new(),
            })),
        }))
    }

    fn has_seen(&self, content: &str) -> bool {
        self.inner.read().unwrap().seen.contains(content)
    }
}

impl Clone for MockMessageProcessor {
    fn clone(&self) -> Self {
        MockMessageProcessor {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl MessageProcessor for MockMessageProcessor {
    fn process_incoming_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message.payload {
            TestMessage(content) => {
                self.inner.write().unwrap().seen.insert(content);
                Ok(())
            }
        }
    }

    fn clone_box(&self) -> Box<dyn MessageProcessor> {
        Box::new(self.clone())
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

    {
        let proc_guard = processor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!proc_guard.has_seen("Hello, World!"));
    }
    
    {
        let proc_guard = processor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(mock_network
            .register_processor(proc_guard.clone_box())
            .is_ok());
    }
    
    assert!(hub.route_message(message).is_ok());
    
    {
        let proc_guard = processor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(proc_guard.has_seen("Hello, World!"));
    }
}

/// This test ensures correct routing and processing of messages between mock networks through the `NetworkHub`.
#[test]
fn test_hub_route_message() {
    let hub = NetworkHub::new();

    let id_1 = random_identifier();
    let mock_net_1 = NetworkHub::new_mock_network(hub.clone(), id_1).unwrap();
    let msg_proc_1 = MockMessageProcessor::new();
    {
        let proc_guard = msg_proc_1.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        mock_net_1
            .register_processor(proc_guard.clone_box())
            .expect("Failed to register message processor");
    }

    let id_2 = random_identifier();
    let mock_net_2 = NetworkHub::new_mock_network(hub, id_2).unwrap();

    let message = Message {
        payload: TestMessage("Test message".to_string()),
        target_node_id: id_1,
    };

    {
        let proc_guard = msg_proc_1.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!proc_guard.has_seen("Test message"));
    }
    
    assert!(mock_net_2.send_message(message).is_ok());
    
    {
        let proc_guard = msg_proc_1.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(proc_guard.has_seen("Test message"));
    }
}

/// This test verifies that cloning a NetworkHub results in a shallow copy where cloned instances share the same underlying data.
#[test]
fn test_network_hub_shallow_clone() {
    let hub = NetworkHub::new();
    let hub_clone = (*hub).clone();
    
    let identifier = random_identifier();
    
    // Create a mock network through the original hub
    let mock_network = NetworkHub::new_mock_network(hub.clone(), identifier).unwrap();
    
    // Create a message to route through the cloned hub
    let message = Message {
        payload: TestMessage("Shallow clone test".to_string()),
        target_node_id: identifier,
    };
    
    // Register a processor on the mock network
    let processor = MockMessageProcessor::new();
    {
        let proc_guard = processor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        mock_network
            .register_processor(proc_guard.clone_box())
            .expect("Failed to register message processor");
    }
    
    // Verify the message hasn't been seen yet
    {
        let proc_guard = processor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(!proc_guard.has_seen("Shallow clone test"));
    }
    
    // Route message through the CLONED hub - this should work because it shares the same underlying data
    assert!(hub_clone.route_message(message).is_ok());
    
    // Verify the message was processed - proving the clone shares the same networks map
    {
        let proc_guard = processor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(proc_guard.has_seen("Shallow clone test"));
    }
}

/// This test sends 10 messages concurrently from mock_net_2 to id_1 and verifies that all messages are processed.
#[test]
fn test_concurrent_message_sending() {
    let hub = NetworkHub::new();

    let id_1 = random_identifier();
    let mock_net_1 = NetworkHub::new_mock_network(hub.clone(), id_1).unwrap();
    let msg_proc_1 = MockMessageProcessor::new();
    {
        let proc_guard = msg_proc_1.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        mock_net_1
            .register_processor(proc_guard.clone_box())
            .expect("Failed to register message processor");
    }

    let id_2 = random_identifier();
    let mock_net_2 = NetworkHub::new_mock_network(hub, id_2).unwrap();

    // Create 10 different message contents
    let message_contents: Vec<String> =
        (0..10).map(|i| format!("Concurrent message {i}")).collect();

    // Set up a barrier to synchronize all threads
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    // Spawn 10 threads, each sending a different message
    for content in message_contents.iter() {
        let content = content.clone();
        let barrier_clone = barrier.clone();
        let mock_net_2_clone = mock_net_2.clone();
        let id_1_copy = id_1;

        let handle = thread::spawn(move || {
            let message = Message {
                payload: TestMessage(content),
                target_node_id: id_1_copy,
            };

            // Wait for all threads to reach this point
            barrier_clone.wait();

            // Send the message
            mock_net_2_clone.send_message(message).unwrap();
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify that all messages were received
    let processor = msg_proc_1.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    for content in message_contents {
        assert!(
            processor.has_seen(&content),
            "Message '{content}' was not received"
        );
        println!("Message '{content}' was successfully processed");
    }
}
