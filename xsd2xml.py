import xml.etree.ElementTree as ET
from lxml import etree
from faker import Faker
import random
import re
from datetime import datetime, timedelta
from typing import Dict, Any, Optional, List, Set


class XMLGenerator:
    def __init__(self, schema_path: str):
        self.schema_path = schema_path
        self.fake = Faker()
        self.schema_doc = etree.parse(schema_path)
        self.schema = etree.XMLSchema(self.schema_doc)
        self.target_namespace = self.schema_doc.getroot().get('targetNamespace', '')
        self.elements = {}
        self.complex_types = {}
        self.simple_types = {}
        self.referenced_elements = set()
        self._parse_schema()

    def _parse_schema(self):
        """Parse the XSD schema and extract element, complex type, and simple type definitions."""
        root = self.schema_doc.getroot()
        ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}

        # First, find all referenced elements
        for ref in root.xpath('//xs:element[@ref]', namespaces=ns):
            ref_name = ref.get('ref')
            if ref_name:
                self.referenced_elements.add(ref_name)

        # Extract elements
        for elem in root.xpath('//xs:element[@name]', namespaces=ns):
            name = elem.get('name')
            if name:
                self.elements[name] = elem

        # Extract complex types
        for ct in root.xpath('//xs:complexType[@name]', namespaces=ns):
            name = ct.get('name')
            if name:
                self.complex_types[name] = ct

        # Extract simple types
        for st in root.xpath('//xs:simpleType[@name]', namespaces=ns):
            name = st.get('name')
            if name:
                self.simple_types[name] = st

    def _find_root_element(self) -> str:
        """Find the most appropriate root element."""
        # Root element is typically one that:
        # 1. Is not referenced by other elements
        # 2. Has a complex type (not a simple type)

        potential_roots = []
        ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}

        for name, elem in self.elements.items():
            # Skip if this element is referenced by others
            if name in self.referenced_elements:
                continue

            # Check if it has a complex type or inline complex type definition
            type_attr = elem.get('type', '')
            has_complex_type = (
                    type_attr in self.complex_types or
                    elem.xpath('./xs:complexType', namespaces=ns) or
                    (not type_attr.startswith('xs:') and type_attr != '')
            )

            if has_complex_type or not type_attr:
                potential_roots.append(name)

        # If we found potential roots, return the first one
        if potential_roots:
            return potential_roots[0]

        # Otherwise, return the first element that's not referenced
        for name in self.elements:
            if name not in self.referenced_elements:
                return name

        # If all elements are referenced, just return the first one
        return next(iter(self.elements)) if self.elements else None

    def _get_fake_data_for_type(self, xsd_type: str, restrictions: Dict[str, Any] = None) -> str:
        """Generate fake data based on XSD type and restrictions."""
        restrictions = restrictions or {}

        if xsd_type in ['xs:string', 'string']:
            if 'enumeration' in restrictions:
                return random.choice(restrictions['enumeration'])
            elif 'pattern' in restrictions:
                # Simple pattern matching for common cases
                pattern = restrictions['pattern']
                if r'\d' in pattern:
                    return ''.join([str(random.randint(0, 9)) for _ in range(5)])
                else:
                    return self.fake.word()
            else:
                max_length = restrictions.get('maxLength', 50)
                return self.fake.text(max_nb_chars=min(max_length, 50))

        elif xsd_type in ['xs:int', 'xs:integer', 'int', 'integer']:
            min_val = restrictions.get('minInclusive', 1)
            max_val = restrictions.get('maxInclusive', 1000)
            return str(random.randint(int(min_val), int(max_val)))

        elif xsd_type in ['xs:positiveInteger', 'positiveInteger']:
            min_val = max(1, int(restrictions.get('minInclusive', 1)))
            max_val = restrictions.get('maxInclusive', 1000)
            return str(random.randint(min_val, int(max_val)))

        elif xsd_type in ['xs:decimal', 'xs:double', 'xs:float', 'decimal', 'double', 'float']:
            return str(round(random.uniform(1.0, 1000.0), 2))

        elif xsd_type in ['xs:boolean', 'boolean']:
            return str(random.choice(['true', 'false']))

        elif xsd_type in ['xs:date', 'date']:
            return self.fake.date()

        elif xsd_type in ['xs:dateTime', 'dateTime']:
            return self.fake.date_time().isoformat()

        elif xsd_type in ['xs:time', 'time']:
            return self.fake.time()

        else:
            return self.fake.word()

    def _extract_restrictions(self, element) -> Dict[str, Any]:
        """Extract restrictions from an element or type definition."""
        restrictions = {}
        ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}

        # Look for restrictions in simpleType or complexType
        for restriction in element.xpath('.//xs:restriction', namespaces=ns):
            # Enumerations
            enums = restriction.xpath('.//xs:enumeration/@value', namespaces=ns)
            if enums:
                restrictions['enumeration'] = enums

            # Pattern
            pattern = restriction.xpath('.//xs:pattern/@value', namespaces=ns)
            if pattern:
                restrictions['pattern'] = pattern[0]

            # Length restrictions
            for attr in ['minLength', 'maxLength', 'length']:
                value = restriction.xpath(f'.//xs:{attr}/@value', namespaces=ns)
                if value:
                    restrictions[attr] = int(value[0])

            # Numeric restrictions
            for attr in ['minInclusive', 'maxInclusive', 'minExclusive', 'maxExclusive']:
                value = restriction.xpath(f'.//xs:{attr}/@value', namespaces=ns)
                if value:
                    restrictions[attr] = value[0]

        return restrictions

    def _generate_element(self, element_def, parent_element: ET.Element, depth: int = 0):
        """Generate XML element based on schema definition."""
        if depth > 10:  # Prevent infinite recursion
            return

        ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}

        # Check if this is a reference to another element
        ref = element_def.get('ref')
        if ref:
            # Look up the referenced element
            if ref in self.elements:
                referenced_element = self.elements[ref]
                # Generate the referenced element with its original name
                self._generate_element(referenced_element, parent_element, depth)
            else:
                # If reference not found, create a simple element
                elem = ET.SubElement(parent_element, ref)
                elem.text = self.fake.word()
            return

        name = element_def.get('name')
        if not name:
            return

        type_attr = element_def.get('type', '')

        # Handle minOccurs and maxOccurs
        min_occurs = int(element_def.get('minOccurs', '1'))
        max_occurs = element_def.get('maxOccurs', '1')
        if max_occurs == 'unbounded':
            max_occurs = random.randint(2, 5)  # Generate 2-5 elements for unbounded
        else:
            max_occurs = int(max_occurs)

        # Ensure we generate at least minOccurs and possibly more
        if max_occurs > min_occurs:
            occurs = random.randint(min_occurs, max_occurs)
        else:
            occurs = min_occurs

        for _ in range(occurs):
            elem = ET.SubElement(parent_element, name)

            # Handle attributes
            for attr in element_def.xpath('.//xs:attribute', namespaces=ns):
                attr_name = attr.get('name')
                attr_type = attr.get('type', 'xs:string')
                attr_use = attr.get('use', 'optional')

                if attr_use == 'required' or random.choice([True, False]):
                    restrictions = self._extract_restrictions(attr)
                    attr_value = self._get_fake_data_for_type(attr_type, restrictions)
                    elem.set(attr_name, attr_value)

            # Handle element content
            has_content = False
            if type_attr:
                if type_attr.startswith('xs:') or type_attr in ['string', 'integer', 'double', 'float', 'boolean',
                                                                'date', 'dateTime', 'time', 'positiveInteger']:
                    # Simple type
                    restrictions = self._extract_restrictions(element_def)
                    elem.text = self._get_fake_data_for_type(type_attr, restrictions)
                    has_content = True
                elif type_attr in self.complex_types:
                    # Complex type
                    self._generate_complex_type(self.complex_types[type_attr], elem, depth + 1)
                    has_content = True
                elif type_attr in self.simple_types:
                    # Custom simple type
                    restrictions = self._extract_restrictions(self.simple_types[type_attr])
                    base_type = self.simple_types[type_attr].xpath('.//xs:restriction/@base', namespaces=ns)
                    if base_type:
                        elem.text = self._get_fake_data_for_type(base_type[0], restrictions)
                    else:
                        elem.text = self._get_fake_data_for_type('xs:string', restrictions)
                    has_content = True
            else:
                # Inline type definition
                complex_type = element_def.xpath('./xs:complexType', namespaces=ns)
                simple_type = element_def.xpath('./xs:simpleType', namespaces=ns)

                if complex_type:
                    self._generate_complex_type(complex_type[0], elem, depth + 1)
                    has_content = True
                elif simple_type:
                    restrictions = self._extract_restrictions(simple_type[0])
                    base_type = simple_type[0].xpath('.//xs:restriction/@base', namespaces=ns)
                    if base_type:
                        elem.text = self._get_fake_data_for_type(base_type[0], restrictions)
                    else:
                        elem.text = self._get_fake_data_for_type('xs:string', restrictions)
                    has_content = True

            # Ensure empty elements have explicit closing tags
            if not has_content and not elem.text and len(elem) == 0:
                elem.text = ''

    def _generate_complex_type(self, complex_type_def, parent_element: ET.Element, depth: int = 0):
        """Generate content for a complex type."""
        if depth > 10:  # Prevent infinite recursion
            return

        ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}

        # Handle sequences
        for sequence in complex_type_def.xpath('.//xs:sequence', namespaces=ns):
            for child_elem in sequence.xpath('./xs:element', namespaces=ns):
                self._generate_element(child_elem, parent_element, depth + 1)

        # Handle choices
        for choice in complex_type_def.xpath('.//xs:choice', namespaces=ns):
            child_elements = choice.xpath('./xs:element', namespaces=ns)
            if child_elements:
                chosen_elem = random.choice(child_elements)
                self._generate_element(chosen_elem, parent_element, depth + 1)

        # Handle all
        for all_group in complex_type_def.xpath('.//xs:all', namespaces=ns):
            for child_elem in all_group.xpath('./xs:element', namespaces=ns):
                self._generate_element(child_elem, parent_element, depth + 1)

        # Handle attributes at complex type level
        for attr in complex_type_def.xpath('./xs:attribute', namespaces=ns):
            attr_name = attr.get('name')
            attr_type = attr.get('type', 'xs:string')
            attr_use = attr.get('use', 'optional')

            if attr_use == 'required' or random.choice([True, False]):
                restrictions = self._extract_restrictions(attr)
                attr_value = self._get_fake_data_for_type(attr_type, restrictions)
                parent_element.set(attr_name, attr_value)

    def generate_xml(self, root_element_name: str = None, output_path: str = None) -> str:
        """Generate XML file with fake data."""
        if root_element_name is None:
            # Find the most appropriate root element
            root_element_name = self._find_root_element()
            if not root_element_name:
                raise ValueError("No suitable root element found in schema")

        if root_element_name not in self.elements:
            raise ValueError(f"Root element '{root_element_name}' not found in schema")

        root_def = self.elements[root_element_name]

        # Create the actual root element
        root = ET.Element(root_element_name)
        if self.target_namespace:
            root.set('xmlns', self.target_namespace)

        # Generate content for root element
        type_attr = root_def.get('type', '')
        if type_attr and type_attr in self.complex_types:
            self._generate_complex_type(self.complex_types[type_attr], root)
        else:
            # Handle inline complex type or simple content
            ns = {'xs': 'http://www.w3.org/2001/XMLSchema'}
            complex_type = root_def.xpath('./xs:complexType', namespaces=ns)
            simple_type = root_def.xpath('./xs:simpleType', namespaces=ns)

            if complex_type:
                self._generate_complex_type(complex_type[0], root)
            elif simple_type:
                restrictions = self._extract_restrictions(simple_type[0])
                base_type = simple_type[0].xpath('.//xs:restriction/@base', namespaces=ns)
                if base_type:
                    root.text = self._get_fake_data_for_type(base_type[0], restrictions)
                else:
                    root.text = self._get_fake_data_for_type('xs:string', restrictions)
            elif type_attr:
                # Simple type root element
                restrictions = self._extract_restrictions(root_def)
                root.text = self._get_fake_data_for_type(type_attr, restrictions)

        # Create XML tree and format it
        tree = ET.ElementTree(root)
        ET.indent(tree, space="  ", level=0)

        if output_path:
            tree.write(output_path, encoding='utf-8', xml_declaration=True, method='xml')
            return output_path
        else:
            # Use method='xml' to ensure proper formatting
            return ET.tostring(root, encoding='unicode', method='xml')


def generate_xml_from_schema(schema_path: str, root_element: str = None, output_path: str = None) -> str:
    """
    Generate XML file with fake data based on XSD schema.

    Args:
        schema_path: Path to the XSD schema file
        root_element: Name of the root element (optional, will use first global element if not specified)
        output_path: Path where to save the generated XML file (optional)

    Returns:
        Generated XML as string or path to saved file
    """
    generator = XMLGenerator(schema_path)
    return generator.generate_xml(root_element, output_path)


# Example usage
if __name__ == "__main__":
    import os
    for example in os.listdir("examples"):
        if example.endswith(".xsd"):
            print()
            print(example)
            schema = os.path.join("examples", example)
            result = generate_xml_from_schema(schema)
            print(result)
