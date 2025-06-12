use crate::core::model::direction::Direction;
use crate::core::model::identity::Identity;

/// LookupTableLevel represents level of a lookup table. entry in the table.
pub type LookupTableLevel = usize;

/// LookupTable is the core view of Skip Graph node towards the network.
pub trait LookupTable {
    /// Update the entry at the given level and direction.
    fn update_entry(
        &self,
        identity: Identity,
        level: LookupTableLevel,
        direction: Direction,
    ) -> anyhow::Result<()>;

    /// Remove the entry at the given level and direction.
    fn remove_entry(&self, level: LookupTableLevel, direction: Direction) -> anyhow::Result<()>;

    /// Get the entry at the given level and direction.
    /// Returns None if the entry is not present.
    /// Returns Some(Identity) if the entry is present.
    fn get_entry(
        &self,
        level: LookupTableLevel,
        direction: Direction,
    ) -> anyhow::Result<Option<Identity>>;

    /// Dynamically compares the lookup table with another for equality.
    fn equal(&self, other: &dyn LookupTable) -> bool;

    fn clone_box(&self) -> Box<dyn LookupTable>;
}

impl PartialEq for dyn LookupTable {
    fn eq(&self, other: &dyn LookupTable) -> bool {
        self.equal(other)
    }
}

impl Clone for Box<dyn LookupTable> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
