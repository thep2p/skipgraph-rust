use crate::core::model::identity::Identity;
use crate::core::search::id_search_req::IdentifierSearchRequest;

pub struct IdentifierSearchResult<T> {
    req : IdentifierSearchRequest,
    result: Option<Identity<T>>,
}

impl<T> IdentifierSearchResult<T> {
    pub fn new(req: IdentifierSearchRequest, result: Option<Identity<T>>) -> Self {
        IdentifierSearchResult { req, result }
    }

    pub fn request(&self) -> &IdentifierSearchRequest {
        &self.req
    }

    pub fn result(&self) -> Option<&Identity<T>> {
        self.result.as_ref()
    }
}