use crate::core::{
    Identifier, IdentifierSearchRequest, IdentifierSearchResult, LookupTable,
    MembershipVector, Node,
};

/// LocalNode is a struct that represents a single node in the local implementation of the skip graph.
struct LocalNode<'a> {
    id: Identifier,
    mem_vec: MembershipVector,
    lt: Box<dyn LookupTable<&'a LocalNode<'a>>>,
}

impl<'a> Node for LocalNode<'a> {
    type Address = &'a LocalNode<'a>;

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

/// Implementing PartialEq for LocalNode to compare the id and membership vector.
/// This basically supports == operator for LocalNode.
/// The cardinal assumption is that the id and membership vector are unique for each node.
impl<'a> PartialEq for LocalNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.mem_vec == other.mem_vec
        // ignore lt for equality check as comparing trait objects is non-trivial
    }
}

#[cfg(test)]
mod tests {
    use crate::core::ArrayLookupTable;
    use super::*;
    use crate::core::testutil::fixtures::{random_identifier, random_membership_vector};

    #[test]
    fn test_local_node() {
        let id = random_identifier();
        let mem_vec = random_membership_vector();
        let node = LocalNode {
            id: id.clone(),
            mem_vec: mem_vec.clone(),
            lt: Box::new(ArrayLookupTable::new()),
        };
        assert_eq!(node.get_identifier(), &id);
        assert_eq!(node.get_membership_vector(), &mem_vec);
        assert_eq!(node.get_address(), &node);
    }
}
