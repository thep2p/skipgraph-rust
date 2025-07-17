use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{anyhow, Context};
use crate::core::Identifier;
use crate::network::mock::network::MockNetwork;
use crate::network::{Message, Network};

pub struct NetworkHub {
    networks: RwLock<HashMap<Identifier, Arc<MockNetwork>>>,
}

impl NetworkHub {
    pub fn new() -> Self {
        NetworkHub {
            networks: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_mock_network(self: &Arc<Self>, identifier: Identifier) -> anyhow::Result<Arc<MockNetwork>> {
        let mut inner_networks = self.networks.write().map_err(|e| anyhow!("Failed to acquire write lock on network hub"))?;
        if inner_networks.contains_key(&identifier) {
            return Err(anyhow::anyhow!("Network with identifier {} already exists", identifier));
        }
        let mock_network = Arc::new(MockNetwork::new(self.clone()));
        inner_networks.insert(identifier, mock_network.clone());
        Ok(mock_network)
    }

    pub fn get_network(&self, identifier: &Identifier) -> Option<Arc<MockNetwork>> {
        let inner_networks = self.networks.read().map_err(|_| anyhow!("Failed to acquire read lock on network hub")).ok()?;
        inner_networks.get(identifier).cloned()
    }

    pub fn route_message(&self, message: Message) -> anyhow::Result<()> {
        let inner_networks = self.networks.read().map_err(|e| anyhow!("Failed to acquire read lock on network hub"))?;
        if let Some(network) = inner_networks.get(&message.target_node_id) {
            network.send_message(message).context("Failed to send message through network")?;
            Ok(())
        } else {
            Err(anyhow!("Network with identifier {} not found", message.target_node_id))
        }
    }
}