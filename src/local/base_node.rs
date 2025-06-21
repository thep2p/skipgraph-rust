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
        for lvl in 0..req.level() {
            match self.lt.get_entry(lvl, req.direction()) {
                Ok(opt) => {
                    if let Some(identity) = opt {
                        // Check if the identity matches the requested identifier
                        if identity.id().eq(req.target()) {
                            candidates.push((*identity.id(), lvl));
                        }
                    }
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
    use crate::core::testutil::fixtures::{
        random_identifier, random_lookup_table_with_extremes,
        random_membership_vector, span_fixture,
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
        let lt = random_lookup_table_with_extremes(LOOKUP_TABLE_LEVELS);
        let node = LocalNode {
            id: random_identifier(),
            mem_vec: random_membership_vector(),
            lt: Box::new(lt.clone()),
        };

        for lvl in 0..LOOKUP_TABLE_LEVELS {
            let target = random_identifier();
            let direction = Direction::Left;
            let req = IdentifierSearchRequest::new(target, lvl, direction);

            let actual_result = node.search_by_id(&req).unwrap();
            let left_neighbors = lt.left_neighbors().unwrap();
            let (_, expected_result) = left_neighbors
                .iter()
                .filter(|(lvl, id)| lvl <= &req.level() && id.id() >= req.target())
                .min_by_key(|(id, _)| *id).unwrap();

            assert_eq!(expected_result.id(), actual_result.result());
        }
    }
    //
    // /// Test that returns the correct candidate when searching in the right direction,
    // /// where the greatest identifier less than or equal to the target should be returned.
    // #[test]
    // fn test_search_by_id_found_right_direction() {
    //     todo!()
    // }
    //
    // /// Test that returns the node's own address when no candidates are found matching the target.
    // #[test]
    // fn test_search_by_id_no_candidates() {
    //     todo!()
    // }
    //
    // /// Test that returns an error when the lookup table returns an error during search at any level.
    // #[test]
    // fn test_search_by_id_error_propagation() {
    //     todo!()
    // }
    //
    // /// Test that correctly handles multiple candidates and returns the appropriate candidate
    // /// per direction and identifier comparison logic.
    // #[test]
    // fn test_search_by_id_multiple_candidates() {
    //     todo!()
    // }
}
