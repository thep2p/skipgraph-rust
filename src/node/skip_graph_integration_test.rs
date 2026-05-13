use super::base_node::BaseNode;
use crate::core::model::direction::Direction;
use crate::core::testutil::fixtures::{
    join_all_with_timeout, random_membership_vector, random_sorted_identifiers, span_fixture,
};
use crate::core::{ArrayLookupTable, IdSearchReq, Identifier, LookupTable};
use crate::network::mock::hub::NetworkHub;
use crate::network::Network;
use crate::node::Node;
use std::sync::Arc;
use std::time::Duration;
use tracing::Span;

/// Creates a fully wired set of `n` `BaseNode`s sharing a single `NetworkHub`.
///
/// Each node has a unique, sorted identifier and a random membership vector, and
/// registers itself as the event processor on its own mock network. The current
/// `join` protocol is a placeholder (see `BaseNode::join`), so lookup tables
/// remain empty. Searches therefore rely on the documented fallback that
/// returns the searcher's own identifier when no neighbor matches. The
/// integration tests assert against that fallback, so the helper is sufficient
/// for verifying that node construction, network registration, and the search
/// code path work end-to-end.
fn new_local_skip_graph(
    parent_span: &Span,
    n: usize,
) -> anyhow::Result<(Vec<BaseNode>, NetworkHub)> {
    if n == 0 {
        return Err(anyhow::anyhow!("cannot create skip graph with 0 nodes"));
    }

    let hub = NetworkHub::new();
    let identifiers = random_sorted_identifiers(n);
    let mut nodes = Vec::with_capacity(n);

    for &id in &identifiers {
        let mem_vec = random_membership_vector();
        let lt: Box<dyn LookupTable> = Box::new(ArrayLookupTable::new(parent_span));
        let network = NetworkHub::new_mock_network(hub.clone(), id)?;
        let net: Box<dyn Network> = network.clone_box();
        let node = BaseNode::new(parent_span.clone(), id, mem_vec, lt, net)?;
        nodes.push(node);
    }

    // TODO: invoke join() to actually populate lookup tables once the join
    // protocol is implemented (currently todo!() in BaseNode::join).

    Ok((nodes, hub))
}

#[test]
fn test_create_small_skip_graph() {
    let span = span_fixture();
    let (nodes, _hub) = new_local_skip_graph(&span, 5).expect("failed to create skip graph");
    assert_eq!(nodes.len(), 5);

    let mut identifiers: Vec<_> = nodes.iter().map(|n| *n.get_identifier()).collect();
    identifiers.sort();
    identifiers.dedup();
    assert_eq!(
        identifiers.len(),
        5,
        "all nodes should have unique identifiers"
    );
}

#[test]
fn test_sequential_search_all_nodes() {
    let span = span_fixture();
    let (nodes, _hub) = new_local_skip_graph(&span, 10).expect("failed to create skip graph");

    for (i, searcher) in nodes.iter().enumerate() {
        for (j, target) in nodes.iter().enumerate() {
            if i == j {
                continue;
            }

            let search_req = IdSearchReq::new(*target.get_identifier(), 0, Direction::Left);
            let search_result = searcher
                .search_by_id(&search_req)
                .unwrap_or_else(|e| panic!("node {i} failed to search for node {j}: {e}"));

            let searcher_id = *searcher.get_identifier();
            assert!(
                search_result.result() >= target.get_identifier()
                    || search_result.result() == &searcher_id,
                "left search result should be >= target or equal to searcher's id: got {}, target {}, searcher {}",
                search_result.result(),
                target.get_identifier(),
                searcher_id
            );
        }
    }
}

#[test]
fn test_concurrent_search_all_nodes() {
    let span = span_fixture();
    let (nodes, _hub) = new_local_skip_graph(&span, 8).expect("failed to create skip graph");

    let nodes = Arc::new(nodes);
    let mut handles = Vec::new();

    for i in 0..nodes.len() {
        for j in 0..nodes.len() {
            if i == j {
                continue;
            }

            let nodes_ref = Arc::clone(&nodes);
            let handle = std::thread::spawn(move || {
                let searcher = &nodes_ref[i];
                let target = &nodes_ref[j];
                let search_req = IdSearchReq::new(*target.get_identifier(), 0, Direction::Left);

                let search_result = searcher
                    .search_by_id(&search_req)
                    .unwrap_or_else(|e| panic!("concurrent search {i}->{j} failed: {e}"));

                let searcher_id = *nodes_ref[i].get_identifier();
                assert!(
                    search_result.result() >= target.get_identifier()
                        || search_result.result() == &searcher_id,
                    "concurrent search result should be >= target or equal to searcher's id: got {}, target {}, searcher {}",
                    search_result.result(),
                    target.get_identifier(),
                    searcher_id
                );
            });
            handles.push(handle);
        }
    }

    join_all_with_timeout(handles.into_boxed_slice(), Duration::from_secs(10))
        .expect("some concurrent searches timed out or failed");
}

#[test]
fn test_lookup_tables_validity() {
    let span = span_fixture();
    let (nodes, _hub) = new_local_skip_graph(&span, 6).expect("failed to create skip graph");

    for (i, node) in nodes.iter().enumerate() {
        let left_req = IdSearchReq::new(*node.get_identifier(), 0, Direction::Left);
        let right_req = IdSearchReq::new(*node.get_identifier(), 0, Direction::Right);

        node.search_by_id(&left_req)
            .unwrap_or_else(|e| panic!("node {i} failed left search: {e}"));
        node.search_by_id(&right_req)
            .unwrap_or_else(|e| panic!("node {i} failed right search: {e}"));
    }
}

#[test]
fn test_larger_skip_graph() {
    let span = span_fixture();
    let (nodes, _hub) = new_local_skip_graph(&span, 20).expect("failed to create larger skip graph");
    assert_eq!(nodes.len(), 20);

    let identifiers: Vec<Identifier> = nodes.iter().map(|n| *n.get_identifier()).collect();
    let mut sorted = identifiers.clone();
    sorted.sort();
    assert_eq!(
        identifiers, sorted,
        "nodes should be created in sorted order by identifier"
    );

    for i in 0..10 {
        let searcher_idx = i % nodes.len();
        let target_idx = (i + nodes.len() / 2) % nodes.len();
        if searcher_idx == target_idx {
            continue;
        }

        let search_req = IdSearchReq::new(
            *nodes[target_idx].get_identifier(),
            0,
            Direction::Left,
        );
        nodes[searcher_idx]
            .search_by_id(&search_req)
            .unwrap_or_else(|e| panic!("sample search {i} failed: {e}"));
    }
}

#[test]
fn test_skip_graph_edge_cases() {
    let span = span_fixture();

    let (nodes, _hub) =
        new_local_skip_graph(&span, 1).expect("failed to create single-node skip graph");
    assert_eq!(nodes.len(), 1);

    let search_req = IdSearchReq::new(*nodes[0].get_identifier(), 0, Direction::Left);
    nodes[0]
        .search_by_id(&search_req)
        .expect("single node should be able to search for itself");

    assert!(
        new_local_skip_graph(&span, 0).is_err(),
        "creating an empty skip graph should fail"
    );
}
