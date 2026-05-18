use super::base_node::BaseNode;
use crate::core::model::direction::Direction;
use crate::core::model::identity::Identity;
use crate::core::testutil::fixtures::{
    join_all_with_timeout, random_address, random_identifier, random_identifier_greater_than,
    random_identifier_less_than, random_lookup_table_with_extremes, random_membership_vector,
    span_fixture,
};
use crate::core::{
    ArrayLookupTable, IdSearchReq, Identifier, LookupTable, LookupTableLevel, LOOKUP_TABLE_LEVELS,
};

use crate::network::{Event, EventProcessorCore, NetworkMock};
use crate::node::Node;
use anyhow::anyhow;
use rand::Rng;
use std::sync::Arc;
use unimock::*;

// TODO: move other tests from base_node.rs here
/// Verifies `search_by_id` returns the node itself when its lookup table is empty.
#[test]
fn test_search_by_id_singleton_fallback() {
    // Node with identifier 10 and empty lookup table
    let origin_id = Identifier::from_bytes(&[10u8]).unwrap();
    let origin_mv = random_membership_vector();
    let mock_net = Unimock::new((
        NetworkMock::register_processor
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        NetworkMock::clone_box
            .each_call(matching!())
            .answers(&|mock| Box::new(mock.clone())),
    ));
    let node = BaseNode::new(
        span_fixture(),
        origin_id,
        origin_mv,
        Box::new(ArrayLookupTable::new(&span_fixture())),
        Box::new(mock_net),
    )
    .expect("failed to create BaseNode");

    // Left and right searches for identifiers 5 and 15
    let cases = [
        (Identifier::from_bytes(&[5u8]).unwrap(), Direction::Left),
        (Identifier::from_bytes(&[15u8]).unwrap(), Direction::Left),
        (Identifier::from_bytes(&[5u8]).unwrap(), Direction::Right),
        (Identifier::from_bytes(&[15u8]).unwrap(), Direction::Right),
    ];

    for (target, direction) in cases {
        let req = IdSearchReq::new(origin_id, target, 3, direction);
        let res = node.search_by_id(&req).expect("search failed");
        // Ensures the search is terminated at the level zero.
        assert_eq!(res.termination_level(), 0);
        // Ensures the search result matches the node's identifier; fallback to self
        assert_eq!(*res.result(), origin_id);
    }
}

/// Verifies left-direction search returns the smallest neighbor with identifier >= target.
#[test]
fn test_search_by_id_found_left_direction() {
    for lvl in 0..LOOKUP_TABLE_LEVELS {
        let lt = random_lookup_table_with_extremes(LOOKUP_TABLE_LEVELS);
        let target = random_identifier();

        // Generate a random identifier greater than the target to ensure we have a candidate
        // Puts the candidate in the left direction at zero level
        let safe_neighbor = random_identifier_greater_than(&target);
        lt.update_entry(
            Identity::new(
                &safe_neighbor,
                &random_membership_vector(),
                random_address(),
            ),
            0,
            Direction::Left,
        )
        .expect("failed to update entry in lookup table");

        let mock_net = Unimock::new((
            NetworkMock::register_processor
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
            NetworkMock::clone_box
                .each_call(matching!())
                .answers(&|mock| Box::new(mock.clone())),
        ));

        let node = BaseNode::new(
            span_fixture(),
            random_identifier(),
            random_membership_vector(),
            Box::new(lt.clone()),
            Box::new(mock_net),
        )
        .expect("failed to create BaseNode");

        let direction = Direction::Left;
        let req = IdSearchReq::new(*node.get_identifier(), target, lvl, direction);

        let actual_result = node.search_by_id(&req).unwrap();

        let (expected_lvl, expected_identity) = lt
            .left_neighbors()
            .unwrap()
            .into_iter()
            .filter(|(l, id)| *l <= req.level() && id.id() >= req.target())
            .min_by_key(|(_, id)| *id.id())
            .unwrap();

        assert_eq!(expected_lvl, actual_result.termination_level());
        assert_eq!(*expected_identity.id(), *actual_result.result());
    }
}

/// Verifies right-direction search returns the greatest neighbor with identifier <= target.
#[test]
fn test_search_by_id_found_right_direction() {
    // Iterate through each level and perform a search
    for lvl in 0..LOOKUP_TABLE_LEVELS {
        let lt = random_lookup_table_with_extremes(LOOKUP_TABLE_LEVELS);
        let target = random_identifier();

        // Generate a random identifier less than the target to ensure we have a candidate
        // Puts the candidate in the right direction at zero level
        let safe_neighbor = random_identifier_less_than(&target);
        lt.update_entry(
            Identity::new(
                &safe_neighbor,
                &random_membership_vector(),
                random_address(),
            ),
            0,
            Direction::Right,
        )
        .expect("failed to update entry in lookup table");

        let mock_net = Unimock::new((
            NetworkMock::register_processor
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
            NetworkMock::clone_box
                .each_call(matching!())
                .answers(&|mock| Box::new(mock.clone())),
        ));

        let node = BaseNode::new(
            span_fixture(),
            random_identifier(),
            random_membership_vector(),
            Box::new(lt.clone()),
            Box::new(mock_net),
        )
        .expect("failed to create BaseNode");

        let direction = Direction::Right;
        let req = IdSearchReq::new(*node.get_identifier(), target, lvl, direction);

        let actual_result = node.search_by_id(&req).unwrap();

        let (expected_lvl, expected_identity) = lt
            .right_neighbors()
            .unwrap()
            .into_iter()
            .filter(|(lvl, id)| *lvl <= req.level() && id.id() <= req.target())
            .max_by_key(|(_, id)| *id.id())
            .unwrap();

        assert_eq!(expected_lvl, actual_result.termination_level());
        assert_eq!(*expected_identity.id(), *actual_result.result());
    }
}

/// Verifies left-direction search falls back to the node itself when no neighbor satisfies the target.
#[test]
fn test_search_by_id_not_found_left_direction() {
    let target = random_identifier();

    // Test that returns the node's own address when no candidates are found matching the target in the left direction.
    for lvl in 0..LOOKUP_TABLE_LEVELS {
        let lt = ArrayLookupTable::new(&span_fixture());

        // Populate the left neighbors of the lookup table with entries that are all less than the target
        // This ensures that no candidates are found in the left direction
        for lvl in 0..LOOKUP_TABLE_LEVELS {
            lt.update_entry(
                Identity::new(
                    &random_identifier_less_than(&target),
                    &random_membership_vector(),
                    random_address(),
                ),
                lvl,
                Direction::Left,
            )
            .expect("failed to update entry in lookup table");
        }

        let mock_net = Unimock::new((
            NetworkMock::register_processor
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
            NetworkMock::clone_box
                .each_call(matching!())
                .answers(&|mock| Box::new(mock.clone())),
        ));

        let node = BaseNode::new(
            span_fixture(),
            random_identifier(),
            random_membership_vector(),
            Box::new(lt.clone()),
            Box::new(mock_net),
        )
        .expect("failed to create BaseNode");

        let direction = Direction::Left;
        let req = IdSearchReq::new(*node.get_identifier(), target, lvl, direction);

        let actual_result = node.search_by_id(&req).unwrap();

        assert_eq!(actual_result.termination_level(), 0);
        assert_eq!(*actual_result.result(), *node.get_identifier());
    }
}

/// Verifies right-direction search falls back to the node itself when no neighbor satisfies the target.
#[test]
fn test_search_by_id_not_found_right_direction() {
    let target = random_identifier();

    // Test that returns the node's own address when no candidates are found matching the target in the right direction.
    for lvl in 0..LOOKUP_TABLE_LEVELS {
        let lt = ArrayLookupTable::new(&span_fixture());

        // Populate the right neighbors of the lookup table with entries that are all greater than the target
        // This ensures that no candidates are found in the right direction
        for lvl in 0..LOOKUP_TABLE_LEVELS {
            lt.update_entry(
                Identity::new(
                    &random_identifier_greater_than(&target),
                    &random_membership_vector(),
                    random_address(),
                ),
                lvl,
                Direction::Right,
            )
            .expect("failed to update entry in lookup table");
        }

        let mock_net = Unimock::new((
            NetworkMock::register_processor
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
            NetworkMock::clone_box
                .each_call(matching!())
                .answers(&|mock| Box::new(mock.clone())),
        ));

        let node = BaseNode::new(
            span_fixture(),
            random_identifier(),
            random_membership_vector(),
            Box::new(lt.clone()),
            Box::new(mock_net),
        )
        .expect("failed to create BaseNode");

        let direction = Direction::Right;
        let req = IdSearchReq::new(*node.get_identifier(), target, lvl, direction);

        let actual_result = node.search_by_id(&req).unwrap();

        assert_eq!(actual_result.termination_level(), 0);
        assert_eq!(*actual_result.result(), *node.get_identifier());
    }
}

/// Verifies `search_by_id` returns the exact match when the target exists in the lookup table.
#[test]
fn test_search_by_id_exact_result() {
    let lt = random_lookup_table_with_extremes(LOOKUP_TABLE_LEVELS);

    let mock_net = Unimock::new((
        NetworkMock::register_processor
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        NetworkMock::clone_box
            .each_call(matching!())
            .answers(&|mock| Box::new(mock.clone())),
    ));

    let node = BaseNode::new(
        span_fixture(),
        random_identifier(),
        random_membership_vector(),
        Box::new(lt.clone()),
        Box::new(mock_net),
    )
    .expect("failed to create BaseNode");

    // This test should ensure that when the exact target is found, it returns the correct level and identifier.
    for lvl in 0..LOOKUP_TABLE_LEVELS {
        for direction in [Direction::Left, Direction::Right] {
            let target_identity = lt.get_entry(lvl, direction).unwrap().unwrap();
            let target = target_identity.id();
            let req = IdSearchReq::new(*node.get_identifier(), *target, lvl, direction);

            let actual_result = node.search_by_id(&req).unwrap();

            assert_eq!(actual_result.termination_level(), lvl);
            assert_eq!(*actual_result.result(), *target);
        }
    }
}

/// Verifies left-direction `search_by_id` returns correct results under concurrent access from 20 threads.
#[test]
fn test_search_by_id_concurrent_found_left_direction() {
    let lt = random_lookup_table_with_extremes(LOOKUP_TABLE_LEVELS);
    let target = random_identifier();

    // 1-liner with TRUE shallow cloning + full mocking features!
    let mock_net = Unimock::new((
        NetworkMock::register_processor
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        NetworkMock::clone_box
            .each_call(matching!())
            .answers(&|mock| Box::new(mock.clone())),
    ));

    let node = BaseNode::new(
        span_fixture(),
        random_identifier(),
        random_membership_vector(),
        Box::new(lt.clone()),
        Box::new(mock_net),
    )
    .expect("failed to create BaseNode");

    // Ensure the target is not the same as the node's identifier
    assert_ne!(&target, node.get_identifier());

    // Spawn 20 threads to perform concurrent searches
    let num_threads = 20;
    let barrier = Arc::new(std::sync::Barrier::new(num_threads + 1)); // +1 for the main thread
    let mut handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
    for _ in 0..num_threads {
        let handle_barrier = barrier.clone();
        let node_ref = node.clone();
        let lt_clone = lt.clone();
        let handle = std::thread::spawn(move || {
            // Wait for all threads to be ready
            handle_barrier.wait();

            // Pick a random level for the search
            let lvl = rand::rng().random_range(0..LOOKUP_TABLE_LEVELS);

            // Perform the search in the left direction
            let req = IdSearchReq::new(*node_ref.get_identifier(), target, lvl, Direction::Left);
            let actual_result = node_ref.search_by_id(&req).unwrap();

            let expected_result = lt_clone
                .left_neighbors()
                .unwrap()
                .into_iter()
                .filter(|(l, id)| *l <= req.level() && id.id() >= req.target())
                .min_by_key(|(_, id)| *id.id());

            match expected_result {
                Some((expected_lvl, expected_identity)) => {
                    assert_eq!(expected_lvl, actual_result.termination_level());
                    assert_eq!(*expected_identity.id(), *actual_result.result());
                }
                None => {
                    // If no expected result, it should return its own identifier
                    assert_eq!(actual_result.termination_level(), 0);
                    assert_eq!(*actual_result.result(), *node_ref.get_identifier());
                }
            }
        });
        handles.push(handle);
    }

    // Ensures all threads are ready to run before the main thread tries to join them
    // avoiding a situation where the main thread tries to join thread that haven't started yet.
    barrier.wait();
    let timeout = std::time::Duration::from_millis(1000);
    join_all_with_timeout(handles.into_boxed_slice(), timeout).unwrap();
}

/// Verifies right-direction `search_by_id` returns correct results under concurrent access from 20 threads.
#[test]
fn test_search_by_id_concurrent_right_direction() {
    let lt = random_lookup_table_with_extremes(LOOKUP_TABLE_LEVELS);
    let target = random_identifier();

    let mock_net = Unimock::new((
        NetworkMock::register_processor
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        NetworkMock::clone_box
            .each_call(matching!())
            .answers(&|mock| Box::new(mock.clone())),
    ));

    let node = BaseNode::new(
        span_fixture(),
        random_identifier(),
        random_membership_vector(),
        Box::new(lt.clone()),
        Box::new(mock_net),
    )
    .expect("failed to create BaseNode");

    // Ensure the target is not the same as the node's identifier
    assert_ne!(&target, node.get_identifier());

    // Spawn 20 threads to perform concurrent searches
    let num_threads = 20;
    let barrier = Arc::new(std::sync::Barrier::new(num_threads + 1)); // +1 for the main thread
    let mut handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
    for _ in 0..num_threads {
        let handle_barrier = barrier.clone();
        let node_ref = node.clone();
        let lt_clone = lt.clone();
        let handle = std::thread::spawn(move || {
            // Wait for all threads to be ready
            handle_barrier.wait();

            // Pick a random level for the search
            let lvl = rand::rng().random_range(0..LOOKUP_TABLE_LEVELS);

            // Perform the search in the right direction
            let req = IdSearchReq::new(*node_ref.get_identifier(), target, lvl, Direction::Right);
            let actual_result = node_ref.search_by_id(&req).unwrap();

            let expected_result = lt_clone
                .right_neighbors()
                .unwrap()
                .into_iter()
                .filter(|(l, id)| *l <= req.level() && id.id() <= req.target())
                .max_by_key(|(_, id)| *id.id());

            match expected_result {
                Some((expected_lvl, expected_identity)) => {
                    assert_eq!(expected_lvl, actual_result.termination_level());
                    assert_eq!(*expected_identity.id(), *actual_result.result());
                }
                None => {
                    // If no expected result, it should return its own identifier
                    assert_eq!(actual_result.termination_level(), 0);
                    assert_eq!(*actual_result.result(), *node_ref.get_identifier());
                }
            }
        });
        handles.push(handle);
    }
    // Ensures all threads are ready to run before the main thread tries to join them
    // avoiding a situation where the main thread tries to join thread that haven't started yet.
    barrier.wait();
    let timeout = std::time::Duration::from_millis(1000);
    join_all_with_timeout(handles.into_boxed_slice(), timeout).unwrap();
}

/// Verifies `search_by_id` propagates errors raised by the underlying lookup table.
#[test]
fn test_search_by_id_error_propagation() {
    // Create a mock lookup table that returns an error for specific lookup operations
    struct MockErrorLookupTable;

    impl Clone for MockErrorLookupTable {
        fn clone(&self) -> Self {
            MockErrorLookupTable
        }
    }

    impl LookupTable for MockErrorLookupTable {
        fn update_entry(
            &self,
            _identity: Identity,
            _level: usize,
            _direction: Direction,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        fn remove_entry(&self, _: LookupTableLevel, _: Direction) -> anyhow::Result<()> {
            todo!()
        }

        fn get_entry(&self, _: usize, _: Direction) -> anyhow::Result<Option<Identity>> {
            Err(anyhow!("simulated lookup table error"))
        }

        fn equal(&self, _: &dyn LookupTable) -> bool {
            todo!()
        }

        fn left_neighbors(&self) -> anyhow::Result<Vec<(usize, Identity)>> {
            Ok(Vec::new())
        }

        fn right_neighbors(&self) -> anyhow::Result<Vec<(usize, Identity)>> {
            Ok(Vec::new())
        }

        fn clone_box(&self) -> Box<dyn LookupTable> {
            Box::new(self.clone())
        }
    }

    let mock_net = Unimock::new((
        NetworkMock::register_processor
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        NetworkMock::clone_box
            .each_call(matching!())
            .answers(&|mock| Box::new(mock.clone())),
    ));

    // Create a base node with the mock error lookup table
    let node = BaseNode::new(
        span_fixture(),
        random_identifier(),
        random_membership_vector(),
        Box::new(MockErrorLookupTable),
        Box::new(mock_net),
    )
    .expect("failed to create BaseNode");

    // Create a random search request (any search request will return an error as
    // the mock lookup table is designed to fail)
    let req = IdSearchReq::new(
        *node.get_identifier(),
        random_identifier(),
        3,
        Direction::Left,
    );

    // Execute the search and verify that an error is returned
    let result = node.search_by_id(&req);

    // The search should fail with an error message that includes our simulated error
    assert!(
        result.is_err(),
        "expected an error but got a success result"
    );

    // Check that the error message contains the expected text ("error while searching by id in level")
    // This error message is constructed in the search_by_id method
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("error while searching by id in level"),
        "error message '{error_msg}' doesn't contain expected text"
    );

    // Additionally, check that the error message contains the simulated lookup table error ("simulated lookup table error")
    // This ensures that the error from the lookup table is propagated correctly
    assert!(
        error_msg.contains("simulated lookup table error"),
        "error message '{error_msg}' doesn't contain expected text"
    );
}

/// Verifies the node, acting as an `EventProcessor`, relays an `IdSearchRequest` event to the expected neighbor via the network.
#[test]
fn test_search_by_id_networking_integration_relay() {
    let lt = random_lookup_table_with_extremes(LOOKUP_TABLE_LEVELS);
    let target = random_identifier();

    // Generate a random identifier greater than the target to ensure we have a candidate
    // Puts the candidate in the left direction at zero level
    let safe_neighbor = random_identifier_greater_than(&target);
    lt.update_entry(
        Identity::new(
            &safe_neighbor,
            &random_membership_vector(),
            random_address(),
        ),
        0,
        Direction::Left,
    )
    .expect("failed to update entry in lookup table");

    let node_id = random_identifier();

    // Create the search request event
    let search_request = IdSearchReq::new(node_id, target, 0, Direction::Left);
    let request_event = Event::IdSearchRequest(search_request);

    let (expected_lvl, expected_identity) = lt
        .left_neighbors()
        .unwrap()
        .into_iter()
        .filter(|(l, id)| *l <= search_request.level() && id.id() >= search_request.target())
        .min_by_key(|(_, id)| *id.id())
        .unwrap();

    let mock_net = Unimock::new((
        NetworkMock::register_processor
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        NetworkMock::send_event
            .each_call(matching!(_))
            .answers_arc(Arc::new(
                move |_, id: Identifier, event: Event| match event {
                    Event::IdSearchRequest(req) => {
                        assert_eq!(req.level(), expected_lvl);
                        assert_eq!(id, *expected_identity.id());
                        Ok(())
                    }
                    _ => panic!("expected IdSearchResponse payload, got: {:?}", event),
                },
            ))
            .once(),
        NetworkMock::clone_box
            .each_call(matching!())
            .answers(&|mock| Box::new(mock.clone())),
    ));

    // Create the BaseNode with mock network
    let node = BaseNode::new(
        span_fixture(),
        node_id,
        random_membership_vector(),
        Box::new(lt.clone()),
        Box::new(mock_net),
    )
    .expect("failed to create BaseNode");

    // Process the request event directly through the node's EventProcessorCore implementation
    let origin_id = random_identifier();
    node.process_incoming_event(origin_id, request_event)
        .expect("failed to process request event");
}
