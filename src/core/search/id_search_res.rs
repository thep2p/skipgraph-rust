use crate::core::Identifier;
use crate::core::lookup::lookup_table::LookupTableLevel;
use crate::core::model::identity::Identity;
use crate::core::search::id_search_req::IdentifierSearchRequest;

pub struct IdentifierSearchResult<T> {
    target: Identifier,
    level: LookupTableLevel,
    address: T,
}

impl<T> IdentifierSearchResult<T> {
    pub fn new(target : Identifier, level: LookupTableLevel, address: T) -> Self {
        IdentifierSearchResult {
            target,
            level,
            address,
        }
    }

    pub fn target(&self) -> &Identifier {
        &self.target
    }

    pub fn level(&self) -> LookupTableLevel {
        self.level
    }

    pub fn result(&self) -> &T {
        &self.address
    }
}