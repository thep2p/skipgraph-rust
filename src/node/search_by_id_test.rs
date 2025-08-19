use std::sync::Arc;
use rand::Rng;
use super::base_node::BaseNode;
use crate::core::model::direction::Direction;
use crate::core::testutil::fixtures::{join_all_with_timeout, random_address, random_identifier, random_identifier_greater_than, random_identifier_less_than, random_lookup_table_with_extremes, random_membership_vector, span_fixture};
use crate::core::{ArrayLookupTable, Identifier, IdentifierSearchRequest, LookupTable, LookupTableLevel, LOOKUP_TABLE_LEVELS};
use crate::core::model::identity::Identity;
use crate::node::Node;

// TODO: move other tests from base_node.rs here
/// Tests fallback behavior of `search_by_id` when no neighbors exist.
/// Each case mirrors a search on a singleton node as described in the behavior
/// matrix of issue https://github.com/thep2p/skipgraph-rust/issues/22.
#[test]
fn test_search_by_id_singleton_fallback() {
    // Node with identifier 10 and empty lookup table
    let id = Identifier::from_bytes(&[10u8]).unwrap();
    let mem_vec = random_membership_vector();
    let node = BaseNode::new(
        id,
        mem_vec,
        Box::new(ArrayLookupTable::new(&span_fixture())),
    );

    // Left and right searches for identifiers 5 and 15
    let cases = [
        (Identifier::from_bytes(&[5u8]).unwrap(), Direction::Left),
        (Identifier::from_bytes(&[15u8]).unwrap(), Direction::Left),
        (Identifier::from_bytes(&[5u8]).unwrap(), Direction::Right),
        (Identifier::from_bytes(&[15u8]).unwrap(), Direction::Right),
    ];

    for (target, direction) in cases {
        let req = IdentifierSearchRequest::new(target, 3, direction);
        let res = node.search_by_id(&req).expect("search failed");
        // Ensures the search is terminated at the level zero.
        assert_eq!(res.termination_level(), 0);
        // Ensures the search result matches the node's identifier; fallback to self
        assert_eq!(*res.result(), id);
    }
}
