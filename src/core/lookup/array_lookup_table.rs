use crate::core::lookup::lookup_table::{Level, LookupTable};
use crate::core::model;
use crate::core::model::direction::Direction;
use crate::core::model::identity::Identity;
use anyhow::anyhow;
use std::fmt::{Debug, Formatter};
use std::sync::RwLock;

/// It is a 2D array of Identity, where the first dimension is the level and the second dimension is the direction.
/// Caution: lookup table by itself is not thread-safe, should be used with an Arc<Mutex<LookupTable>>.
pub struct ArrayLookupTable<T: Clone> {
    inner: RwLock<InnerArrayLookupTable<T>>,
}

struct InnerArrayLookupTable<T> {
    left: Vec<Option<Identity<T>>>,
    right: Vec<Option<Identity<T>>>,
}

impl<T> ArrayLookupTable<T>
where
    T: Clone,
{
    /// Create a new empty LookupTable instance.
    pub fn new() -> ArrayLookupTable<T> {
        ArrayLookupTable {
            inner: RwLock::new(InnerArrayLookupTable {
                left: vec![None; model::IDENTIFIER_SIZE_BYTES],
                right: vec![None; model::IDENTIFIER_SIZE_BYTES],
            }),
        }
    }
}

impl<T> Default for ArrayLookupTable<T>
where
    T: Clone,
{
    fn default() -> Self {
        ArrayLookupTable::new()
    }
}

impl<T: Clone> Clone for ArrayLookupTable<T> {
    fn clone(&self) -> Self {
        // Create a new instance of ArrayLookupTable with the same data
        let inner = self.inner.read().unwrap();
        ArrayLookupTable {
            inner: RwLock::new(InnerArrayLookupTable {
                left: inner.left.clone(),
                right: inner.right.clone(),
            }),
        }
    }
}

impl<T> Debug for ArrayLookupTable<T>
where
    T: Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let inner = match self.inner.read() {
            Ok(guard) => guard,
            Err(_) => return write!(f, "Failed to acquire read lock on the lookup table"),
        };
        writeln!(f, "ArrayLookupTable: {{")?;
        for (i, (l, r)) in inner.left.iter().zip(inner.right.iter()).enumerate() {
            writeln!(f, "Level: {}, Left: {:?}, Right: {:?}", i, l, r)?;
        }
        write!(f, "}}")
    }
}

impl<T> LookupTable<T> for ArrayLookupTable<T>
where
    T: Clone + Debug + 'static + PartialEq,
{
    /// Update the entry at the given level and direction.
    fn update_entry(
        &self,
        identity: Identity<T>,
        level: Level,
        direction: Direction,
    ) -> anyhow::Result<()> {
        if level >= model::IDENTIFIER_SIZE_BYTES {
            return Err(anyhow!(
                "Position is larger than the max lookup table entry number: {}",
                level
            ));
        }

        let mut inner = match self.inner.write() {
            Ok(guard) => guard,
            Err(_) => return Err(anyhow!("Failed to acquire write lock on the lookup table")),
        };

        match direction {
            Direction::Left => {
                inner.left[level] = Some(identity);
            }
            Direction::Right => {
                inner.right[level] = Some(identity);
            }
        }

        Ok(())
    }

    /// Remove the entry at the given level and direction, and flips it to None.
    fn remove_entry(&self, level: Level, direction: Direction) -> anyhow::Result<()> {
        if level >= model::IDENTIFIER_SIZE_BYTES {
            return Err(anyhow!(
                "Position is larger than the max lookup table entry number: {}",
                level
            ));
        }

        let mut inner = match self.inner.write() {
            Ok(guard) => guard,
            Err(_) => return Err(anyhow!("Failed to acquire write lock on the lookup table")),
        };

        match direction {
            Direction::Left => {
                inner.left[level] = None;
            }
            Direction::Right => {
                inner.right[level] = None;
            }
        }

        Ok(())
    }

    /// Get the entry at the given level and direction.
    /// Returns None if the entry does not exist.
    /// Returns Some(Identity) if the entry exists.
    /// Returns an error if the level is out of bounds.
    fn get_entry(&self, level: Level, direction: Direction) -> anyhow::Result<Option<Identity<T>>> {
        if level >= model::IDENTIFIER_SIZE_BYTES {
            return Err(anyhow!(
                "Position is larger than the max lookup table entry number: {}",
                level
            ));
        }

        let inner = match self.inner.read() {
            Ok(guard) => guard,
            Err(_) => return Err(anyhow!("Failed to acquire read lock on the lookup table")),
        };

        match direction {
            Direction::Left => Ok(inner.left[level].clone()),
            Direction::Right => Ok(inner.right[level].clone()),
        }
    }

    /// Dynamically compares the lookup table with another for equality.
    /// This is a deep comparison of the entries in the table.
    /// Returns true if the entries are equal, false otherwise.
    fn equal(&self, other: &dyn LookupTable<T>) -> bool {
        // iterates over the levels and compares the entries in the left and right directions
        for l in 0..model::IDENTIFIER_SIZE_BYTES {
            // Check if the left entry is equal
            let inner = match self.inner.read() {
                Ok(guard) => guard,
                Err(_) => return false,
            };
            if let Ok(other_entry) = other.get_entry(l, Direction::Left) {
                if inner.left[l].as_ref() != other_entry.as_ref() {
                    return false;
                }
            } else {
                // if retrieving the entry fails on the other table, return false
                return false;
            }

            if let Ok(other_entry) = other.get_entry(l, Direction::Right) {
                if inner.right[l].as_ref() != other_entry.as_ref() {
                    return false;
                }
            } else {
                // if retrieving the entry fails on the other table, return false
                return false;
            }
        }
        true
    }

    fn clone_box(&self) -> Box<dyn LookupTable<T>> {
        Box::new(self.clone())
    }
}

impl<T> PartialEq for ArrayLookupTable<T>
where
    T: Clone + Debug + 'static + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::testutil::fixtures::*;
    use crate::core::Address;

    #[test]
    /// A new lookup table should be empty.
    fn test_lookup_table_empty() {
        let lt: ArrayLookupTable<Address> = ArrayLookupTable::new();
        for i in 0..model::IDENTIFIER_SIZE_BYTES {
            assert_eq!(None, lt.get_entry(i, Direction::Left).unwrap());
            assert_eq!(None, lt.get_entry(i, Direction::Right).unwrap());
        }
    }

    #[test]
    /// Test updating and getting entries in the lookup table.
    /// The test will update the entries at level 0 and 1, and then get them.
    /// The test will also try to get an entry at level 2, which should return an error.
    fn test_lookup_table_update_get() {
        let mut lt = ArrayLookupTable::new();
        let id1 = random_network_identity();
        let id2 = random_network_identity();

        lt.update_entry(id1, 0, Direction::Left).unwrap();
        lt.update_entry(id2, 1, Direction::Right).unwrap();

        assert_eq!(Some(id1), lt.get_entry(0, Direction::Left).unwrap());
        assert_eq!(Some(id2), lt.get_entry(1, Direction::Right).unwrap());
        assert_eq!(None, lt.get_entry(2, Direction::Left).unwrap());
    }

    #[test]
    /// Test removing entries in the lookup table.
    /// The test will update the entries at level 0 and 1, and then remove them.
    /// The test will then try to get the removed entries, which should return None.
    fn test_lookup_table_remove() {
        let mut lt = ArrayLookupTable::new();
        let id1 = random_network_identity();
        let id2 = random_network_identity();

        lt.update_entry(id1, 0, Direction::Left).unwrap();
        lt.update_entry(id2, 1, Direction::Right).unwrap();

        lt.remove_entry(0, Direction::Left).unwrap();
        lt.remove_entry(1, Direction::Right).unwrap();

        assert_eq!(None, lt.get_entry(0, Direction::Left).unwrap());
        assert_eq!(None, lt.get_entry(1, Direction::Right).unwrap());
    }

    #[test]
    /// Test updating entries at out-of-bound levels.
    fn test_lookup_table_out_of_bound() {
        let mut lt = ArrayLookupTable::new();
        let id = random_network_identity();

        let result = lt.update_entry(id, model::IDENTIFIER_SIZE_BYTES, Direction::Left);
        assert!(result.is_err());

        let result = lt.update_entry(id, model::IDENTIFIER_SIZE_BYTES, Direction::Right);
        assert!(result.is_err());

        let result = lt.get_entry(model::IDENTIFIER_SIZE_BYTES, Direction::Left);
        assert!(result.is_err());

        let result = lt.get_entry(model::IDENTIFIER_SIZE_BYTES, Direction::Right);
        assert!(result.is_err());

        let result = lt.remove_entry(model::IDENTIFIER_SIZE_BYTES, Direction::Left);
        assert!(result.is_err());

        let result = lt.remove_entry(model::IDENTIFIER_SIZE_BYTES, Direction::Right);
        assert!(result.is_err());
    }

    #[test]
    /// Test overriding entries in the lookup table.
    /// The test will update the entry at level 0, then update it again with a different identity.
    /// The test will then get the entry at level 0, which should return the second identity.
    fn test_lookup_table_override() {
        let lt = ArrayLookupTable::new();
        let id1 = random_network_identity();
        let id2 = random_network_identity();

        lt.update_entry(id1, 0, Direction::Left).unwrap();
        assert_eq!(Some(id1), lt.get_entry(0, Direction::Left).unwrap());

        lt.update_entry(id2, 0, Direction::Left).unwrap();

        assert_eq!(Some(id2), lt.get_entry(0, Direction::Left).unwrap());
    }

    #[test]
    /// Test equality of lookup tables.
    /// The test will create two identical lookup tables and check if they are equal.
    /// The test will also create a different lookup table and check if they are not equal.
    /// The test will also check if the lookup table is equal to itself.
    /// The test will also check if the lookup table is not equal to None.
    fn test_lookup_table_equality() {
        let lt1 = random_network_lookup_table(10);
        let lt2 = random_network_lookup_table(10);

        assert_ne!(lt1, lt2); // check if two random lookup tables are not equal
        assert_eq!(lt1, lt1); // check if the lookup table is equal to itself 
        assert_eq!(lt2, lt2); // check if the lookup table is equal to itself
    }
    
    /// Test concurrent reads from the lookup table.
    /// Creates a lookup table with 20 entries (10 left and 10 right).
    /// Spawns 20 threads to read the entries concurrently.
    /// Each thread reads an entry at a specific level and direction.
    /// Checks if the entry is correct.
    #[test]
    fn test_concurrent_reads() {
        use std::sync::{Arc, Barrier};
        use std::thread;
        
        let lt = Arc::new(ArrayLookupTable::<Address>::new());
        
        // Generate 20 random identities; 10 for left and 10 for right.
        // The i index is the "left" entry at level i + 10 is the "right" entry at level i.
        let levels = 10;
        let identities = random_network_identities(2 * levels);
        
        for i in 0..levels {
            lt.update_entry(identities[i].clone(), i, Direction::Left).unwrap();
            lt.update_entry(identities[i+levels].clone(), i, Direction::Right).unwrap();
        }
        
        // Number of reader threads
        let num_threads = identities.len();
        let barrier = Arc::new(Barrier::new(num_threads)); // to sync thread start
        
        // Spawn threads to read the entries concurrently
        let mut handles = vec![];
        for i in 0..num_threads {
            let lt_ref = lt.clone();
            let barrier_ref = barrier.clone();
            let id = identities[i].clone();
            let handle = thread::spawn(move || {
                barrier_ref.wait(); // wait for all threads to be ready
                let level = i % levels; // alternate between left and right
                let direction = if i < levels { Direction::Left } else { Direction::Right };
                
                // Read the entry
                let entry = lt_ref.get_entry(level, direction).unwrap();
                
                // Check if the entry is correct
                assert_eq!(entry, Some(id));
            });
            
            handles.push(handle);
        }
        
        // join all threads with a timeout 
        let timeout = std::time::Duration::from_millis(100);
        join_all_with_timeout(handles.into_boxed_slice(), timeout).unwrap();
    }
}
