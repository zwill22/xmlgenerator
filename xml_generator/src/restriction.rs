pub struct RestrictionInfo {
    pub(crate) name: String,
    pub(crate) facets: Vec<String>,
}

impl RestrictionInfo {
    pub(crate) fn new() -> RestrictionInfo {
        RestrictionInfo {
            name: String::new(),
            facets: Vec::new(),
        }
    }
}
