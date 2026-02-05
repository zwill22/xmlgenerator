use crate::XMLGeneratorError;
use crate::namespaces::Namespaces;
use xsd_parser::models::schema::{QName, SchemaInfo};

#[derive(Default)]
pub(crate) struct Name {
    name: String,
    namespace: Option<String>,
}

impl Name {
    pub(crate) fn new(name: String, namespace: Option<String>) -> Self {
        Self { name, namespace }
    }

    pub(crate) fn from_qname(qname: &QName, namespaces: &Namespaces) -> Self {
        let name = String::from_utf8(qname.local_name().to_vec()).unwrap();
        let ns = match qname.namespace() {
            None => None,
            Some(namespace) => namespaces
                .find(&namespace.to_string())
                .map(|ns_name| ns_name.to_string()),
        };

        Self::new(name, ns)
    }

    pub(crate) fn from_name(
        name: &Option<String>,
        schema: &SchemaInfo,
        namespaces: &Namespaces,
    ) -> Option<Name> {
        match &schema.schema.target_namespace {
            None => name.as_ref().map(|val| Name::new(val.clone(), None)),
            Some(ns) => match name {
                None => None,
                Some(name) => match namespaces.find(ns) {
                    None => Some(Name::new(name.clone(), None)),
                    Some(ns_prefix) => Some(Name::new(name.clone(), Some(ns_prefix.clone()))),
                },
            },
        }
    }

    pub(crate) fn get_suffix(&self) -> Result<String, XMLGeneratorError> {
        if self.name.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Empty name".to_string(),
            ));
        }

        Ok(self.name.clone())
    }

    pub(crate) fn get_name(&self) -> Result<String, XMLGeneratorError> {
        if self.name.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Name is empty".to_string(),
            ));
        }

        let output = match &self.namespace {
            Some(ns) => {
                if ns.is_empty() {
                    return Err(XMLGeneratorError::DataTypesFormatError(
                        "Namespace is empty".to_string(),
                    ));
                }

                format!("{}:{}", ns, self.name)
            }
            None => self.get_suffix()?,
        };

        Ok(output)
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        if !self.name.eq(other.name.as_str()) {
            return false;
        }

        match &self.namespace {
            None => other.namespace.is_none(),
            Some(ns) => match &other.namespace {
                None => false,
                Some(other_ns) => ns.eq(other_ns),
            },
        }
    }
}
