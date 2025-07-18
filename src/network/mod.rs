pub mod mock;

use std::any::Any;
use std::sync::{Arc, Mutex};
use crate::core::Identifier;

/// MessageType enum defines the types of messages that can be sent over the network.
#[derive(Debug)]
pub enum MessageType {
    TestMessage(String), // A message for testing purposes, it is a simple string message, and is not used in production.
}

/// Message struct represents a message that can be sent over the network.
pub struct Message {
    pub message_type: MessageType,
    pub target_node_id: Identifier,
    pub payload: Box<dyn Any + Send>
}

/// MessageProcessor trait defines the entity that processes the incoming network messages at this node.
pub trait MessageProcessor: Send + Sync {
    fn process_incoming_message(&mut self, message: Message) -> anyhow::Result<()>;
}

/// Network trait defines the interface for a network service that can send and receive messages.
pub trait Network: Send + Sync {
    /// Sends a message to the network.
    fn send_message(&self, message: Message) -> anyhow::Result<()>;

    /// Registers a message processor to handle incoming messages. 
    /// At any point in time, there can be only one processor registered.
    /// Registering a new processor is illegal if there is already a processor registered, and causes an error.
    fn register_processor(&mut self, processor: Box<Arc<Mutex<dyn MessageProcessor>>>) -> anyhow::Result<()>;

    /// Starts the network service.
    fn start(&mut self) -> anyhow::Result<()>;

    /// Stops the network service.
    fn stop(&mut self) -> anyhow::Result<()>;
}