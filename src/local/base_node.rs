use crate::core::{
    Identifier, IdentifierSearchRequest, IdentifierSearchResult, LookupTable,
    MembershipVector, Node,
};

/// LocalNode is a struct that represents a single node in the local implementation of the skip graph.
struct LocalNode {
    id: Identifier,
    mem_vec: MembershipVector,
    lt: dyn LookupTable<&LocalNode>,
}

impl<'a> Node for LocalNode {
    type Address = &'a LocalNode;

    fn get_identifier(&self) -> &Identifier {
        &self.id
    }

    fn get_membership_vector(&self) -> &MembershipVector {
        &self.mem_vec
    }

    fn get_address(&self) -> Self::Address {
        &self
    }

    fn search_by_id(
        &self,
        req: &IdentifierSearchRequest,
    ) -> anyhow::Result<IdentifierSearchResult<Self::Address>> {
        todo!()
    }

    fn search_by_mem_vec(
        &self,
        req: &IdentifierSearchRequest,
    ) -> anyhow::Result<IdentifierSearchResult<Self::Address>> {
        todo!()
    }

    fn join(&self, introducer: Self::Address) -> anyhow::Result<()> {
        todo!()
    }
}
