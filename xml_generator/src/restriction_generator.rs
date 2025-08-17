pub struct RestrictionGenerator {
    pub(crate) name: String,
    pub(crate) facets: Vec<String>,
}

impl RestrictionGenerator {
    pub(crate) fn new() -> RestrictionGenerator {
        RestrictionGenerator {
            name: String::new(),
            facets: Vec::new(),
        }
    }
}
