use crate::XMLGeneratorError;
use crate::namespaces::Namespaces;
use xsd_parser::models::schema::{QName, SchemaInfo};

fn validate(name: &str) -> Result<(), XMLGeneratorError> {
    const VALID: &str = r"^[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}][-.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]*$";

    let regex = regex::Regex::new(VALID).unwrap();

    if !regex.is_match(name) {
        return Err(XMLGeneratorError::InvalidXSDNameError(name.to_string()));
    }

    Ok(())
}

#[derive(Default)]
pub(crate) struct Name {
    name: String,
    namespace: Option<String>,
}

impl Name {
    fn new(name: String, namespace: Option<String>) -> Result<Self, XMLGeneratorError> {
        validate(&name)?;
        if let Some(ns) = &namespace {
            validate(ns)?;
        }

        let out = Self { name, namespace };

        Ok(out)
    }

    pub(crate) fn from_qname(
        qname: &QName,
        namespaces: &Namespaces,
    ) -> Result<Self, XMLGeneratorError> {
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
    ) -> Result<Option<Name>, XMLGeneratorError> {
        let name_val = match name {
            None => return Ok(None),
            Some(val) => val,
        };

        let prefix = Self::get_target_ns(schema, namespaces);
        let name = Name::new(name_val.to_string(), prefix)?;

        Ok(Some(name))
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
