use crate::core::Identifier;
use crate::core::lookup::lookup_table::LookupTableLevel;
use crate::core::model::identity::Identity;
use crate::core::search::id_search_req::IdentifierSearchRequest;

pub struct IdentifierSearchResult<T> {
    target: Identifier,
    level: LookupTableLevel,
    result: Option<Identity<T>>,
}

impl<T> IdentifierSearchResult<T> {
    pub fn new(req: IdentifierSearchRequest, result: Option<Identity<T>>) -> Self {
        IdentifierSearchResult {
            target: req.target,
            level: req.level,
            result,
        }
    }

    pub fn target(&self) -> &Identifier {
        &self.target
    }

    pub fn level(&self) -> LookupTableLevel {
        self.level
    }

    pub fn result(&self) -> Option<&Identity<T>> {
        self.result.as_ref()
    }
}