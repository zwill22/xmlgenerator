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

    fn get_target_ns(schema_info: &SchemaInfo, namespaces: &Namespaces) -> Option<String> {
        match &schema_info.schema.target_namespace {
            None => None,
            Some(ns) => namespaces.find(ns).map(|prefix| prefix.to_string()),
        }
    }

    pub(crate) fn from_name(
        schema: &SchemaInfo,
        namespaces: &Namespaces,
        name: &Option<String>,
    ) -> Option<Name> {
        let name_val = match name {
            None => return None,
            Some(val) => val,
        };

        let prefix = Self::get_target_ns(schema, namespaces);

        Some(Name::new(name_val.to_string(), prefix))
    }

    pub(crate) fn get_prefix(&self) -> Option<String> {
        self.namespace.clone()
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
