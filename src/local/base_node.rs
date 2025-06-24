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
    use crate::core::testutil::fixtures::{
        random_identifier, random_lookup_table_with_extremes, random_membership_vector,
        span_fixture,
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

            let expected = lt
                .left_neighbors()
                .unwrap()
                .into_iter()
                .filter(|(l, id)| *l <= req.level && id.id() >= req.target())
                .min_by_key(|(_, id)| *id.id());

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
    
    // /// Test that returns the node's own address when no candidates are found matching the target.
    // #[test]
    // fn test_search_by_id_no_candidates() {
    //     let lt = ArrayLookupTable::new(&span_fixture());
    //     let node = LocalNode {
    //         id: random_identifier(),
    //         mem_vec: random_membership_vector(),
    //         lt: Box::new(lt),
    //     };
    // 
    //     let target = random_identifier();
    //     let direction = Direction::Left;
    //     let req = IdentifierSearchRequest::new(target, , direction);
    // 
    //     let res = node.search_by_id(&req).unwrap();
    //     assert_eq!(*node.get_identifier(), *res.result());
    // }
    //
    // /// Test that returns an error when the lookup table returns an error during search at any level.
    // #[test]
    // fn test_search_by_id_error_propagation() {
    //     struct FaultyLookupTable;
    //     impl LookupTable for FaultyLookupTable {
    //         fn update_entry(
    //             &self,
    //             _identity: Identity,
    //             _level: usize,
    //             _direction: Direction,
    //         ) -> anyhow::Result<()> {
    //             Ok(())
    //         }
    //
    //         fn remove_entry(&self, _level: usize, _direction: Direction) -> anyhow::Result<()> {
    //             Ok(())
    //         }
    //
    //         fn get_entry(
    //             &self,
    //             _level: usize,
    //             _direction: Direction,
    //         ) -> anyhow::Result<Option<Identity>> {
    //             Err(anyhow::anyhow!("lookup failure"))
    //         }
    //
    //         fn equal(&self, _other: &dyn LookupTable) -> bool {
    //             false
    //         }
    //
    //         fn left_neighbors(&self) -> anyhow::Result<Vec<(usize, Identity)>> {
    //             Ok(vec![])
    //         }
    //
    //         fn right_neighbors(&self) -> anyhow::Result<Vec<(usize, Identity)>> {
    //             Ok(vec![])
    //         }
    //
    //         fn clone_box(&self) -> Box<dyn LookupTable> {
    //             Box::new(FaultyLookupTable)
    //         }
    //     }
    //
    //     let node = LocalNode {
    //         id: random_identifier(),
    //         mem_vec: random_membership_vector(),
    //         lt: Box::new(FaultyLookupTable),
    //     };
    //
    //     let req = IdentifierSearchRequest::new(random_identifier(), 3, Direction::Left);
    //     let result = node.search_by_id(&req);
    //     assert!(result.is_err());
    // }
    //
    // /// Test that correctly handles multiple candidates and returns the appropriate candidate
    // /// per direction and identifier comparison logic.
    // #[test]
    // fn test_search_by_id_multiple_candidates() {
    //     let lt = ArrayLookupTable::new(&span_fixture());
    //
    //     // Create deterministic identifiers for clarity
    //     let id1 = Identifier::from_bytes(&[10]).unwrap();
    //     let id2 = Identifier::from_bytes(&[20]).unwrap();
    //     let id3 = Identifier::from_bytes(&[30]).unwrap();
    //     let mv = random_membership_vector();
    //     let addr = random_address();
    //
    //     lt.update_entry(Identity::new(&id1, &mv, addr), 0, Direction::Right)
    //         .unwrap();
    //     lt.update_entry(Identity::new(&id2, &mv, addr), 1, Direction::Right)
    //         .unwrap();
    //     lt.update_entry(Identity::new(&id3, &mv, addr), 2, Direction::Right)
    //         .unwrap();
    //
    //     let node = LocalNode {
    //         id: random_identifier(),
    //         mem_vec: mv,
    //         lt: Box::new(lt),
    //     };
    //
    //     let target = Identifier::from_bytes(&[25]).unwrap();
    //     let req = IdentifierSearchRequest::new(target, 3, Direction::Right);
    //     let result = node.search_by_id(&req).unwrap();
    //
    //     assert_eq!(id2, *result.result());
    // }
}
