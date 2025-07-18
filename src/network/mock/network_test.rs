use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use crate::network::{Message, MessageProcessor};
use crate::network::MessageType::TestMessage;
use crate::network::mock::network::MockNetwork;

struct MockMessageProcessor {
    seen : HashSet<String>,
    net: Arc<Mutex<MockNetwork>>,
}

impl MockMessageProcessor {
    fn new(net: Arc<Mutex<MockNetwork>>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new((MockMessageProcessor {
            seen: HashSet::new(),
            net,
        })))
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
            },
            _ => {
                Err(anyhow::anyhow!(format!("Unknown message type {:?}", message.message_type)))
            }
        }
    }
}

#[cfg(test)]
mod network_test {
    use crate::core::testutil::fixtures::random_identifier;
    use crate::network::mock::hub::NetworkHub;
    use crate::network::Network;
    use super::*;

    /// Test for the MockMessageProcessor to ensure it correctly processes messages.
    #[test]
    fn test_mock_message_processor() {
        let hub = Arc::new(NetworkHub::new());
        let identifier = random_identifier();
        let mock_network = hub.new_mock_network(identifier).unwrap();
        let mut processor = MockMessageProcessor::new(mock_network.clone());
        let message = Message {
            message_type: TestMessage("Hello, World!".to_string()),
            target_node_id: random_identifier(),
            payload: Box::new(()),
        };

        assert!(!processor.lock().unwrap().has_seen("Hello, World!"));
        processor.lock().unwrap().process_incoming_message(message).unwrap();
        assert!(processor.lock().unwrap().has_seen("Hello, World!"));
    }

    #[test]
    fn test_hub_route_message() {
        use crate::network::mock::hub::NetworkHub;

        let hub = Arc::new(NetworkHub::new());
        
        let id_1 = random_identifier();
        let mock_net_1 = hub.new_mock_network(id_1).unwrap();
        let msg_proc_1 = MockMessageProcessor::new(mock_net_1.clone());
        mock_net_1.lock().unwrap().register_processor(Box::new(msg_proc_1.clone())).expect("Failed to \
        register message processor");

        let id_2 = random_identifier();
        let mock_net_2 = hub.new_mock_network(id_2).unwrap();

        
        let message = Message {
            message_type: TestMessage("Test message".to_string()),
            target_node_id: id_1.clone(),
            payload: Box::new(()),
        };

        assert!(!msg_proc_1.lock().unwrap().has_seen("Test message"));
        assert!(mock_net_2.lock().unwrap().send_message(message).is_ok());
    }
}