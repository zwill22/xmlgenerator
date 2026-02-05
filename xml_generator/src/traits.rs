use crate::XMLGeneratorError;
use crate::element::Element;
use crate::generator::Generator;
use crate::tracker::RecursionTracker;
use crate::xsd::XSD;
use std::slice::Iter;
use xml_builder::XMLElement;

pub(crate) trait GenerateGroups {
    fn elements(&self) -> Iter<'_, Element>;

    fn generate_group(
        &self,
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        tracker: &mut RecursionTracker,
        xsd: &XSD,
    ) -> Result<(), XMLGeneratorError> {
        for element in self.elements() {
            let children = element.generate(generator, tracker, xsd)?;

            for child in children {
                xml_element.add_child(child)?;
            }
        }

        Ok(())
    }
}
