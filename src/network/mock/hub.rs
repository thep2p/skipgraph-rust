use std::cell::RefCell;
use crate::core::Identifier;
use crate::network::mock::network::MockNetwork;
use crate::network::{Message, Network};
use anyhow::{anyhow, Context};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub struct NetworkHub {
    networks: RwLock<HashMap<Identifier, Arc<Mutex<MockNetwork>>>>,
}

impl NetworkHub {
    pub fn new() -> Self {
        NetworkHub {
            networks: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_mock_network(self: &Arc<Self>, identifier: Identifier, ) -> anyhow::Result<Rc<RefCell<MockNetwork>>> {
        let mut inner_networks = self
            .networks
            .write()
            .map_err(|e| anyhow!("Failed to acquire write lock on network hub"))?;
        if inner_networks.contains_key(&identifier) {
            return Err(anyhow::anyhow!(
                "Network with identifier {} already exists",
                identifier
            ));
        }
        let mock_network = Arc::new(Mutex::new(MockNetwork::new(self.clone())));
        inner_networks.insert(identifier, mock_network.clone());
        Ok(mock_network)
    }

    pub fn get_network(&self, identifier: &Identifier) -> Option<Arc<Mutex<MockNetwork>>> {
        let inner_networks = self
            .networks
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock on network hub"))
            .ok()?;
        inner_networks.get(identifier).cloned()
    }

    pub fn route_message(&self, message: Message) -> anyhow::Result<()> {
        let inner_networks = self
            .networks
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock on network hub"))?;
        if let Some(mutex_network) = inner_networks.get(&message.target_node_id) {
            let network = mutex_network
                .lock()
                .map_err(|e| anyhow!("Failed to acquire lock on network {e}"))?;
            network
                .send_message(message)
                .context("Failed to send message through network")?;
            Ok(())
        } else {
            Err(anyhow!(
                "Network with identifier {} not found",
                message.target_node_id
            ))
        }
    }
}
