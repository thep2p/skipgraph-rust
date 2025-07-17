use std::collections::HashSet;
use std::sync::Arc;
use crate::network::{Message, MessageProcessor};
use crate::network::MessageType::TestMessage;
use crate::network::mock::network::MockNetwork;

struct MockMessageProcessor {
    seen : HashSet<String>,
    net: Arc<MockNetwork>,
}

impl MockMessageProcessor {
    fn new(net: Arc<MockNetwork>) -> Self {
        MockMessageProcessor {
            seen: HashSet::new(),
            net,
        }
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
    use super::*;

    /// Test for the MockMessageProcessor to ensure it correctly processes messages.
    #[test]
    fn test_mock_message_processor() {
        let hub = NetworkHub::new();
        let identifier = random_identifier();
        let mock_network = hub.new_mock_network(identifier).unwrap();
        let mut processor = MockMessageProcessor::new(mock_network.clone());
        let message = Message {
            message_type: TestMessage("Hello, World!".to_string()),
            target_node_id: random_identifier(),
            payload: Box::new(()),
        };

        assert!(!processor.has_seen("Hello, World!"));
        processor.process_incoming_message(message).unwrap();
        assert!(processor.has_seen("Hello, World!"));
    }
    
    fn test_hub_route_message() {
        use crate::network::mock::hub::NetworkHub;

        let hub = NetworkHub::new();
        
        let id_1 = random_identifier();
        let mock_network_1 = hub.get_network(&id_1).unwrap();
        let id_2 = random_identifier();
        let mock_network_2 = hub.get_network(&id_2).unwrap();
        
        let message = Message {
            message_type: TestMessage("Test message".to_string()),
            target_node_id: id_1.clone(),
            payload: Box::new(()),
        };

        // Route the message through the hub
        assert!(hub.route_message(message).is_ok());

        // Verify that the network can be retrieved from the hub
        assert!(hub.get_network(&id_1).is_some());
    }
}