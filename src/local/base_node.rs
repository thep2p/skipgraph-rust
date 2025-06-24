use crate::core::model::direction::Direction;
use crate::core::{
    Identifier, IdentifierSearchRequest, IdentifierSearchResult, LookupTable, MembershipVector,
    Node,
};
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

/// LocalNode is a struct that represents a single node in the local implementation of the skip graph.
pub(crate) struct LocalNode {
    id: Identifier,
    mem_vec: MembershipVector,
    lt: Box<dyn LookupTable>,
}

impl Node for LocalNode {
    type Address = Rc<LocalNode>;

    fn get_identifier(&self) -> &Identifier {
        &self.id
    }

    fn get_membership_vector(&self) -> &MembershipVector {
        &self.mem_vec
    }

    fn get_address(&self) -> Self::Address {
        Rc::new(self.clone())
    }

    fn search_by_id(
        &self,
        req: &IdentifierSearchRequest,
    ) -> anyhow::Result<IdentifierSearchResult> {
        // Collect neighbors from levels <= req.level in req.direction
        let mut candidates = Vec::new();
        for lvl in 0..=req.level() {
            match self.lt.get_entry(lvl, req.direction()) {
                Ok(Some(identity)) => {
                    candidates.push((*identity.id(), lvl));
                }
                Ok(None) => {
                    // No entry found at this level, continue to the next level
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Error while searching by id in level {}: {}",
                        lvl,
                        e
                    ));
                }
            }
        }

        // Filter candidates based on the direction
        let result = match req.direction() {
            Direction::Left => {
                // In the left direction, the result is the smallest identifier that is greater than or equal to the target
                candidates
                    .into_iter()
                    .filter(|(id, _)| id >= req.target())
                    .min_by_key(|(id, _)| *id)
            }
            Direction::Right => {
                // In the right direction, the result is the greatest identifier that is less than or equal to the target
                candidates
                    .into_iter()
                    .filter(|(id, _)| id <= req.target())
                    .max_by_key(|(id, _)| *id)
            }
        };

        match result {
            Some((id, level)) => {
                // If a candidate is found, return it
                Ok(IdentifierSearchResult::new(*req.target(), level, id))
            }
            None => {
                // If no candidates are found, return its own identifier
                Ok(IdentifierSearchResult::new(
                    *req.target(),
                    0,
                    *self.get_identifier(),
                ))
            }
        }
    }

    fn search_by_mem_vec(
        &self,
        _req: &IdentifierSearchRequest,
    ) -> anyhow::Result<IdentifierSearchResult> {
        todo!()
    }

    fn join(&self, _introducer: Self::Address) -> anyhow::Result<()> {
        todo!()
    }
}

/// Implementing PartialEq for LocalNode to compare the id and membership vector.
/// This basically supports == operator for LocalNode.
/// The cardinal assumption is that the id and membership vector are unique for each node.
impl PartialEq for LocalNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.mem_vec == other.mem_vec
        // ignore lt for equality check as comparing trait objects is non-trivial
    }
}

impl fmt::Debug for LocalNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalNode")
            .field("id", &self.id)
            .field("mem_vec", &self.mem_vec)
            .finish()
    }
}

impl Clone for LocalNode {
    fn clone(&self) -> Self {
        LocalNode {
            id: self.id,
            mem_vec: self.mem_vec,
            lt: self.lt.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::identity::Identity;
    use crate::core::testutil::fixtures::{
        random_address, random_identifier, random_identifier_greater_than,
        random_lookup_table_with_extremes, random_membership_vector, span_fixture,
    };
    use crate::core::{ArrayLookupTable, LOOKUP_TABLE_LEVELS};

    #[test]
    fn test_local_node() {
        let id = random_identifier();
        let mem_vec = random_membership_vector();
        let node = LocalNode {
            id,
            mem_vec,
            lt: Box::new(ArrayLookupTable::new(&span_fixture())),
        };
        assert_eq!(node.get_identifier(), &id);
        assert_eq!(node.get_membership_vector(), &mem_vec);
        // TODO: implement get_address for LocalNode
        // assert_eq!(node.get_address(), &node);
    }

    /// Test that returns the correct candidate when searching in the left direction,
    /// where the smallest identifier greater than or equal to the target should be returned.
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
            .expect("Failed to update entry in lookup table");

            let node = LocalNode {
                id: random_identifier(),
                mem_vec: random_membership_vector(),
                lt: Box::new(lt.clone()),
            };

            let direction = Direction::Left;
            let req = IdentifierSearchRequest::new(target, lvl, direction);

            let actual_result = node.search_by_id(&req).unwrap();

            let (expected_lvl, expected_identity) = lt
                .left_neighbors()
                .unwrap()
                .into_iter()
                .filter(|(l, id)| *l <= req.level && id.id() >= req.target())
                .min_by_key(|(_, id)| *id.id())
                .unwrap();

            assert_eq!(expected_lvl, actual_result.level());
            assert_eq!(*expected_identity.id(), *actual_result.result());
        }
    }

    /// Test that returns the correct candidate when searching in the right direction,
    /// where the greatest identifier less than or equal to the target should be returned.
    #[test]
    fn test_search_by_id_found_right_direction() {
        let lt = random_lookup_table_with_extremes(LOOKUP_TABLE_LEVELS);
        let node = LocalNode {
            id: random_identifier(),
            mem_vec: random_membership_vector(),
            lt: Box::new(lt.clone()),
        };

        // Iterate through each level and perform a search
        for lvl in 0..LOOKUP_TABLE_LEVELS {
            let target = random_identifier();
            let direction = Direction::Right;
            let req = IdentifierSearchRequest::new(target, lvl, direction);

            let actual_result = node.search_by_id(&req).unwrap();

            let expected = lt
                .right_neighbors()
                .unwrap()
                .into_iter()
                .filter(|(lvl, id)| *lvl <= req.level() && id.id() <= req.target())
                .max_by_key(|(_, id)| *id.id());

            match expected {
                Some((expected_lvl, expected_identity)) => {
                    assert_eq!(expected_lvl, actual_result.level());
                    assert_eq!(*expected_identity.id(), *actual_result.result());
                }
                None => {
                    assert_eq!(0, actual_result.level());
                    assert_eq!(*node.get_identifier(), *actual_result.result());
                }
            }
        }
    }

    // TODO: test that returns the node's own address when no candidates are found matching the target (left/right direction).
    // TODO: test that returns an error when the lookup table returns an error during search at any level.
    // TODO: test that when the exact target is found, it returns the correct level and identifier.
    // TODO: concurrent tests for search_by_id to ensure thread safety and correctness under concurrent access.
}
