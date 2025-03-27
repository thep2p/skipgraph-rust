use crate::core::{Identifier, IdentifierSearchRequest, IdentifierSearchResult, MembershipVector};

/// Node is a trait that represents a single node in a skip graph.
pub trait Node {
    /// The associated type for the address of the node, in the local implementation, the address is a reference to the node itself,
    /// and in network implementation, the address is the network address of the node.
    type Address;

    /// Returns the identifier of the node.
    fn get_identifier(&self) -> &Identifier;

    /// Returns the membership vector of the node.
    fn get_membership_vector(&self) -> &MembershipVector;

    /// Returns the address of the node, as the node is local, its address is a reference to itself.
    fn get_address(&self) -> Self::Address;

    /// Performs a search for the given identifier in the lookup table in the given direction and level.
    fn search_by_id(&self, req: &IdentifierSearchRequest) -> anyhow::Result<IdentifierSearchResult<Self::Address>>;

    /// Performs a search for the given membership vector in the lookup table in the given direction and level.
    fn search_by_mem_vec(&self, req: &IdentifierSearchRequest) -> anyhow::Result<IdentifierSearchResult<Self::Address>>;
}