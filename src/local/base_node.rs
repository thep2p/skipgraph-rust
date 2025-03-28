use crate::core::{Address, Identifier, IdentifierSearchRequest, IdentifierSearchResult, LookupTable, MembershipVector, Node};
use crate::local::node::LocalNode;

struct BaseNode {
    id: Identifier,
    mem_vec: MembershipVector,
    lt : dyn LookupTable<&BaseNode>,
}

impl<'a> Node for BaseNode {
    type Address = &'a BaseNode;

    fn get_identifier(&self) -> &Identifier {
        &self.id
    }

    fn get_membership_vector(&self) -> &MembershipVector {
        &self.mem_vec
    }

    fn get_address(&self) -> Self::Address {
        &self
    }

    fn search_by_id(&self, req: &IdentifierSearchRequest) -> anyhow::Result<IdentifierSearchResult<Self::Address>> {
        todo!()
    }

    fn search_by_mem_vec(&self, req: &IdentifierSearchRequest) -> anyhow::Result<IdentifierSearchResult<Self::Address>> {
        todo!()
    }

    fn join(&self, introducer: Self::Address) -> anyhow::Result<()> {
        todo!()
    }
}