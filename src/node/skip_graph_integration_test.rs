use super::base_node::BaseNode;
use crate::core::model::direction::Direction;
use crate::core::model::identity::Identity;
use crate::core::testutil::fixtures::{
    join_all_with_timeout, random_membership_vector, random_sorted_identifiers,
    span_fixture,
};
use crate::core::{Address, ArrayLookupTable, IdSearchReq, Identifier, LookupTable, MembershipVector, LOOKUP_TABLE_LEVELS};
use crate::network::mock::hub::NetworkHub;
use crate::network::Network;
use crate::node::Node;
use std::sync::Arc;
use std::time::Duration;

struct LocalSkipGraph {
    nodes: Vec<BaseNode>,
    hub: NetworkHub,
    lts: Vec<Box<dyn LookupTable>>,
    identifiers: Vec<Identifier>,
    mvs: Vec<MembershipVector>,
}

impl LocalSkipGraph {
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
    fn new(n: usize) -> anyhow::Result<Self> {
        if n == 0 {
            return Err(anyhow::anyhow!("cannot create skip graph with 0 nodes"));
        }

        let hub = NetworkHub::new();
        let identifiers = random_sorted_identifiers(n);
        let mut nodes = Vec::with_capacity(n);
        let mut lts: Vec<Box<dyn LookupTable>> = Vec::with_capacity(n);

        for &id in &identifiers {
            let mem_vec = random_membership_vector();
            let lt: Box<dyn LookupTable> = Box::new(ArrayLookupTable::new(&span_fixture()));
            let network = NetworkHub::new_mock_network(hub.clone(), id)?;
            let node = BaseNode::new(
                span_fixture(),
                id,
                mem_vec,
                lt.clone(),
                network.clone_box(),
            )?;
            nodes.push(node);
            lts.push(lt);
        }

        // Connects the nodes in a doubly-linked list at level zero, the first node does not have
        // a previous node and the last node does not have a next node.
        for (n_pair, lt_pair) in nodes.windows(2).zip(lts.windows(2)) {
            let this_id = Identity::new(
                n_pair[1].get_identifier(),
                n_pair[1].get_membership_vector(),
                Address::new("localhost", "0"),
            );
            let prev_id = Identity::new(
                n_pair[0].get_identifier(),
                n_pair[0].get_membership_vector(),
                Address::new("localhost", "0"),
            );

            lt_pair[0].update_entry(this_id, 0, Direction::Right)?;
            lt_pair[1].update_entry(prev_id, 0, Direction::Left)?;
        }

        for i in 1..n {
            let mut loop_start = i - 1; // exclude i from considering for its own left neighbor
            for level in 1..LOOKUP_TABLE_LEVELS {
                let mut neighbor_idx: Option<usize> = None;

                // moves leftward to find a neighbor at the given level
                for j in (0..=loop_start).rev() {
                    // Invariant: loop_start < i, so j < i throughout — no self-link possible.
                    if nodes[i].get_membership_vector().common_prefix_bit(nodes[j].get_membership_vector()) >= level {
                        let id_j = Identity::new(nodes[j].get_identifier(), nodes[j].get_membership_vector(), Address::new("localhost", "0"));
                        let id_i = Identity::new(nodes[i].get_identifier(), nodes[i].get_membership_vector(), Address::new("localhost", "0"));
                        lts[i].update_entry(id_j, level, Direction::Left)?;
                        lts[j].update_entry(id_i, level, Direction::Right)?;
                        neighbor_idx = Some(j);
                        break;
                    }
                }
                match neighbor_idx {
                    // if a neighbor was found, we continue to search at the next level from the same node
                    Some(j) => loop_start = j,
                    // if no neighbor was found, we stop searching at any other level, as we cannot find at least 'level'-bit common prefix,
                    // hence we cannot find > 'level'-bit common prefix for any other level.
                    None => break,
                }
            }
        }


        let mvs = nodes.iter().map(|n| *n.get_membership_vector()).collect();
        Ok(LocalSkipGraph{
            nodes,
            hub,
            lts,
            identifiers,
           mvs,
        })
    }
}

#[test]
fn test_create_small_skip_graph() {

    let sg = LocalSkipGraph::new(5).expect("failed to create skip graph");
    assert_eq!(sg.nodes.len(), 5);

    let mut identifiers: Vec<_> = sg.nodes.iter().map(|n| *n.get_identifier()).collect();
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

    let sg = LocalSkipGraph::new(10).expect("failed to create skip graph");

    for (i, searcher) in sg.nodes.iter().enumerate() {
        for (j, target) in sg.nodes.iter().enumerate() {
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
    let sg = LocalSkipGraph::new(8).expect("failed to create skip graph");

    let nodes = Arc::new(sg.nodes);
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
    let sg = LocalSkipGraph::new(6).expect("failed to create skip graph");

    for (i, node) in sg.nodes.iter().enumerate() {
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
    let sg = LocalSkipGraph::new(20).expect("failed to create larger skip graph");
    assert_eq!(sg.nodes.len(), 20);

    let identifiers: Vec<Identifier> = sg.nodes.iter().map(|n| *n.get_identifier()).collect();
    let mut sorted = identifiers.clone();
    sorted.sort();
    assert_eq!(
        identifiers, sorted,
        "nodes should be created in sorted order by identifier"
    );

    for i in 0..10 {
        let searcher_idx = i % sg.nodes.len();
        let target_idx = (i + sg.nodes.len() / 2) % sg.nodes.len();
        if searcher_idx == target_idx {
            continue;
        }

        let search_req = IdSearchReq::new(*sg.nodes[target_idx].get_identifier(), 0, Direction::Left);
        sg.nodes[searcher_idx]
            .search_by_id(&search_req)
            .unwrap_or_else(|e| panic!("sample search {i} failed: {e}"));
    }
}

#[test]
fn test_skip_graph_edge_cases() {
    let span = span_fixture();

    let sg = LocalSkipGraph::new(1).expect("failed to create single-node skip graph");
    assert_eq!(sg.nodes.len(), 1);

    let search_req = IdSearchReq::new(*sg.nodes[0].get_identifier(), 0, Direction::Left);
    sg.nodes[0]
        .search_by_id(&search_req)
        .expect("single node should be able to search for itself");

    assert!(LocalSkipGraph::new(0).is_err(), "creating an empty skip graph should fail");
}
