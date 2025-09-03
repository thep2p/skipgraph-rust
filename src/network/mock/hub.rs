use crate::core::Identifier;
use crate::network::mock::network::MockNetwork;
use crate::network::{Event};
use anyhow::{anyhow, Context};
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// NetworkHub is a central hub that manages multiple mock networks.
/// It allows for the creation of new mock networks and routing events between them.
/// Events are routed completely through the hub in an in-memory fashion, simulating a network environment without actual network communication.
///
/// Thread-safety is handled internally using RwLock for the networks map, following a Go-like approach
/// where the struct can be safely shared via Arc<NetworkHub> without external locking.
///
/// Implements shallow cloning where cloned instances share the same underlying data.
pub struct NetworkHub {
    networks: Arc<RwLock<HashMap<Identifier, Arc<MockNetwork>>>>,
}

impl NetworkHub {
    pub fn new() -> Self {
        NetworkHub {
            networks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new mock network with the given identifier and registers it in the hub.
    pub fn new_mock_network(hub: Self, identifier: Identifier) -> anyhow::Result<Arc<MockNetwork>> {
        let mut networks = hub.networks.write();

        if networks.contains_key(&identifier) {
            return Err(anyhow!(
                "network with identifier {} already exists",
                identifier
            ));
        }

        let mock_network = Arc::new(MockNetwork::new(identifier.clone(), hub.clone()));
        networks.insert(identifier, mock_network.clone());
        Ok(mock_network)
    }

    // TODO: route_event should be a closure that embeds the origin_id.
    /// Routes an event to the appropriate mock network based on the target node identifier.
    pub fn route_event(&self, origin_id: Identifier, target_id: Identifier, event: Event) -> anyhow::Result<()> {
        let networks = self.networks.read();

        if let Some(network) = networks.get(&target_id) {
            network
                .incoming_event(origin_id, event)
                .context("failed to send event through network")?;
            Ok(())
        } else {
            Err(anyhow!(
                "network with identifier {} not found", target_id
            ))
        }
    }
}

impl Clone for NetworkHub {
    fn clone(&self) -> Self {
        NetworkHub {
            networks: Arc::clone(&self.networks),
        }
    }
}
