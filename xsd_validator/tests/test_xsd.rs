#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use xsdvalidator::XSDValidator;
    use rstest::*;

    #[fixture]
    #[once]
    fn validator() -> XSDValidator {
        XSDValidator::new(true)
    }

    #[rstest]
    fn test_xsd(
        validator: &XSDValidator,
        #[base_dir = ".."]
        #[files("examples/working/*.xsd")]
        path: PathBuf,
    ) {
       validator.validate(&path).expect("Validation failed");
    }
}

