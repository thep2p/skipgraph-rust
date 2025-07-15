use super::base_node::LocalNode;
use crate::core::model::direction::Direction;
use crate::core::testutil::fixtures::span_fixture;
use crate::core::{ArrayLookupTable, Identifier, IdentifierSearchRequest, MembershipVector, Node};

/// Tests fallback behavior of `search_by_id` when no neighbors exist.
/// Each case mirrors a search on a singleton node as described in the behavior
/// matrix of issue #??.
#[test]
fn test_search_by_id_singleton_fallback() {
    // Node with identifier 10 and empty lookup table
    let id = Identifier::from_bytes(&[10u8]).unwrap();
    let mem_vec = MembershipVector::from_bytes(&[0u8]).unwrap();
    let node = LocalNode::new(
        id,
        mem_vec,
        Box::new(ArrayLookupTable::new(&span_fixture())),
    );

    let cases = [
        (Identifier::from_bytes(&[5u8]).unwrap(), Direction::Left),
        (Identifier::from_bytes(&[15u8]).unwrap(), Direction::Left),
        (Identifier::from_bytes(&[5u8]).unwrap(), Direction::Right),
        (Identifier::from_bytes(&[15u8]).unwrap(), Direction::Right),
    ];

    for (target, direction) in cases {
        let req = IdentifierSearchRequest::new(target, 3, direction);
        let res = node.search_by_id(&req).expect("search failed");
        assert_eq!(res.termination_level(), 0);
        assert_eq!(*res.result(), id);
    }
}
