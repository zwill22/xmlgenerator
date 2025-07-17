from lxml import etree
import os
from collections import defaultdict


class RelaxNGToXSDConverter:
    """Pure Python RelaxNG to XSD converter"""

    def __init__(self):
        self.xsd_ns = "http://www.w3.org/2001/XMLSchema"
        self.rng_ns = "http://relaxng.org/ns/structure/1.0"
        self.type_counter = 0
        self.named_patterns = {}
        self.generated_types = {}

    def convert(self, relaxng_path, output_path=None):
        """
        Convert RelaxNG schema to XSD

        Args:
            relaxng_path: Path to RelaxNG file
            output_path: Optional output path for XSD

        Returns:
            XSD as string (also writes to file if output_path provided)
        """
        # Parse RelaxNG
        with open(relaxng_path, 'rb') as f:
            rng_tree = etree.parse(f)

        rng_root = rng_tree.getroot()

        # Create XSD root
        xsd_root = etree.Element(
            f"{{{self.xsd_ns}}}schema",
            nsmap={'xs': self.xsd_ns}
        )

        # First pass: collect named patterns
        self._collect_named_patterns(rng_root)

        # Second pass: convert structure
        self._convert_grammar(rng_root, xsd_root)

        # Generate XSD string
        xsd_string = etree.tostring(
            xsd_root,
            pretty_print=True,
            xml_declaration=True,
            encoding='UTF-8'
        ).decode('utf-8')

        # Write to file if requested
        if output_path:
            with open(output_path, 'w', encoding='utf-8') as f:
                f.write(xsd_string)

        return xsd_string

    def _collect_named_patterns(self, rng_elem):
        """Collect all named pattern definitions"""
        for define in rng_elem.xpath('.//rng:define', namespaces={'rng': self.rng_ns}):
            name = define.get('name')
            self.named_patterns[name] = define

    def _convert_grammar(self, rng_elem, xsd_parent):
        """Convert RelaxNG grammar to XSD"""
        # Find start element
        start = rng_elem.find(f'{{{self.rng_ns}}}start')
        if start is not None:
            for child in start:
                self._convert_pattern(child, xsd_parent)
        else:
            # No explicit start, process all top-level elements
            for child in rng_elem:
                if child.tag != f'{{{self.rng_ns}}}define':
                    self._convert_pattern(child, xsd_parent)

    def _convert_pattern(self, rng_elem, xsd_parent):
        """Convert a RelaxNG pattern to XSD"""
        if rng_elem.tag == f'{{{self.rng_ns}}}element':
            return self._convert_element(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}attribute':
            return self._convert_attribute(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}ref':
            return self._convert_ref(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}choice':
            return self._convert_choice(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}optional':
            return self._convert_optional(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}zeroOrMore':
            return self._convert_zero_or_more(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}oneOrMore':
            return self._convert_one_or_more(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}group':
            return self._convert_group(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}interleave':
            # Interleave is approximated as sequence in XSD
            return self._convert_group(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}text':
            return self._convert_text(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}data':
            return self._convert_data(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}value':
            return self._convert_value(rng_elem, xsd_parent)
        elif rng_elem.tag == f'{{{self.rng_ns}}}empty':
            return None  # Empty pattern
        else:
            # Unknown pattern, try to process children
            for child in rng_elem:
                self._convert_pattern(child, xsd_parent)

    def _convert_element(self, rng_elem, xsd_parent):
        """Convert RelaxNG element to XSD element"""
        name = rng_elem.get('name')

        # Create XSD element
        xsd_elem = etree.SubElement(xsd_parent, f"{{{self.xsd_ns}}}element")

        if name:
            xsd_elem.set('name', name)
        else:
            # Handle name class (choice of names)
            name_choice = rng_elem.find(f'{{{self.rng_ns}}}choice')
            if name_choice is not None:
                # For simplicity, take the first name
                first_name = name_choice.find(f'{{{self.rng_ns}}}name')
                if first_name is not None:
                    xsd_elem.set('name', first_name.text)

        # Check if element has simple content or complex content
        children = list(rng_elem)
        has_attributes = any(self._is_attribute_pattern(child) for child in children)
        has_elements = any(self._is_element_pattern(child) for child in children)
        has_text = any(child.tag == f'{{{self.rng_ns}}}text' for child in children)

        if not children:
            # Empty element
            xsd_elem.set('type', 'xs:string')
        elif has_text and not has_elements and not has_attributes:
            # Simple content
            xsd_elem.set('type', 'xs:string')
        elif has_text and has_attributes and not has_elements:
            # Simple content with attributes
            complex_type = etree.SubElement(xsd_elem, f"{{{self.xsd_ns}}}complexType")
            simple_content = etree.SubElement(complex_type, f"{{{self.xsd_ns}}}simpleContent")
            extension = etree.SubElement(simple_content, f"{{{self.xsd_ns}}}extension")
            extension.set('base', 'xs:string')

            for child in children:
                if self._is_attribute_pattern(child):
                    self._convert_pattern(child, extension)
        else:
            # Complex content
            complex_type = etree.SubElement(xsd_elem, f"{{{self.xsd_ns}}}complexType")

            if has_text and has_elements:
                complex_type.set('mixed', 'true')

            # Create sequence for element content
            if has_elements or len(children) > 1:
                sequence = etree.SubElement(complex_type, f"{{{self.xsd_ns}}}sequence")
                content_parent = sequence
            else:
                content_parent = complex_type

            # Process children
            for child in children:
                if self._is_attribute_pattern(child):
                    self._convert_pattern(child, complex_type)
                elif child.tag != f'{{{self.rng_ns}}}text':
                    self._convert_pattern(child, content_parent)

        return xsd_elem

    def _convert_attribute(self, rng_elem, xsd_parent):
        """Convert RelaxNG attribute to XSD attribute"""
        name = rng_elem.get('name')

        xsd_attr = etree.SubElement(xsd_parent, f"{{{self.xsd_ns}}}attribute")
        xsd_attr.set('name', name)

        # Check for data type or value
        data_elem = rng_elem.find(f'{{{self.rng_ns}}}data')
        value_elem = rng_elem.find(f'{{{self.rng_ns}}}value')
        choice_elem = rng_elem.find(f'{{{self.rng_ns}}}choice')

        if data_elem is not None:
            xsd_type = self._convert_data_type(data_elem.get('type', 'string'))
            xsd_attr.set('type', xsd_type)
        elif value_elem is not None:
            # Fixed value
            xsd_attr.set('fixed', value_elem.text)
            xsd_attr.set('type', 'xs:string')
        elif choice_elem is not None:
            # Enumeration
            simple_type = etree.SubElement(xsd_attr, f"{{{self.xsd_ns}}}simpleType")
            restriction = etree.SubElement(simple_type, f"{{{self.xsd_ns}}}restriction")
            restriction.set('base', 'xs:string')

            for value in choice_elem.findall(f'{{{self.rng_ns}}}value'):
                enum = etree.SubElement(restriction, f"{{{self.xsd_ns}}}enumeration")
                enum.set('value', value.text)
        else:
            # Default to string
            xsd_attr.set('type', 'xs:string')

        return xsd_attr

    def _convert_ref(self, rng_elem, xsd_parent):
        """Convert RelaxNG ref to pattern definition"""
        ref_name = rng_elem.get('name')
        if ref_name in self.named_patterns:
            define_elem = self.named_patterns[ref_name]
            for child in define_elem:
                self._convert_pattern(child, xsd_parent)

    def _convert_choice(self, rng_elem, xsd_parent):
        """Convert RelaxNG choice to XSD choice"""
        xsd_choice = etree.SubElement(xsd_parent, f"{{{self.xsd_ns}}}choice")

        for child in rng_elem:
            self._convert_pattern(child, xsd_choice)

        return xsd_choice

    def _convert_optional(self, rng_elem, xsd_parent):
        """Convert RelaxNG optional pattern"""
        # Process children with minOccurs="0"
        for child in rng_elem:
            result = self._convert_pattern(child, xsd_parent)
            if result is not None and 'minOccurs' not in result.attrib:
                result.set('minOccurs', '0')

    def _convert_zero_or_more(self, rng_elem, xsd_parent):
        """Convert RelaxNG zeroOrMore pattern"""
        for child in rng_elem:
            result = self._convert_pattern(child, xsd_parent)
            if result is not None:
                result.set('minOccurs', '0')
                result.set('maxOccurs', 'unbounded')

    def _convert_one_or_more(self, rng_elem, xsd_parent):
        """Convert RelaxNG oneOrMore pattern"""
        for child in rng_elem:
            result = self._convert_pattern(child, xsd_parent)
            if result is not None:
                result.set('minOccurs', '1')
                result.set('maxOccurs', 'unbounded')

    def _convert_group(self, rng_elem, xsd_parent):
        """Convert RelaxNG group to XSD sequence"""
        xsd_sequence = etree.SubElement(xsd_parent, f"{{{self.xsd_ns}}}sequence")

        for child in rng_elem:
            self._convert_pattern(child, xsd_sequence)

        return xsd_sequence

    def _convert_text(self, rng_elem, xsd_parent):
        """Handle text pattern"""
        # Text is handled by parent element
        return None

    def _convert_data(self, rng_elem, xsd_parent):
        """Convert data pattern"""
        # This is typically handled by parent attribute/element
        return None

    def _convert_value(self, rng_elem, xsd_parent):
        """Convert value pattern"""
        # This is typically handled by parent attribute/element
        return None

    def _convert_data_type(self, rng_type):
        """Convert RelaxNG data type to XSD type"""
        type_map = {
            'string': 'xs:string',
            'token': 'xs:token',
            'integer': 'xs:integer',
            'int': 'xs:int',
            'long': 'xs:long',
            'short': 'xs:short',
            'byte': 'xs:byte',
            'decimal': 'xs:decimal',
            'float': 'xs:float',
            'double': 'xs:double',
            'boolean': 'xs:boolean',
            'date': 'xs:date',
            'dateTime': 'xs:dateTime',
            'time': 'xs:time',
            'anyURI': 'xs:anyURI',
            'ID': 'xs:ID',
            'IDREF': 'xs:IDREF',
            'NMTOKEN': 'xs:NMTOKEN',
        }
        return type_map.get(rng_type, 'xs:string')

    def _is_attribute_pattern(self, elem):
        """Check if element is an attribute pattern"""
        return (elem.tag == f'{{{self.rng_ns}}}attribute' or
                (elem.tag in [f'{{{self.rng_ns}}}optional', f'{{{self.rng_ns}}}group'] and
                 any(self._is_attribute_pattern(child) for child in elem)))

    def _is_element_pattern(self, elem):
        """Check if element is an element pattern"""
        return (elem.tag == f'{{{self.rng_ns}}}element' or
                (elem.tag in [f'{{{self.rng_ns}}}optional', f'{{{self.rng_ns}}}choice',
                              f'{{{self.rng_ns}}}group', f'{{{self.rng_ns}}}zeroOrMore',
                              f'{{{self.rng_ns}}}oneOrMore'] and
                 any(self._is_element_pattern(child) for child in elem)))


# Usage example
def convert_relaxng_to_xsd(relaxng_path, xsd_path=None):
    """
    Convert a RelaxNG schema to XSD

    Args:
        relaxng_path: Path to RelaxNG file
        xsd_path: Optional output path (defaults to same name with .xsd extension)

    Returns:
        Path to generated XSD file
    """
    converter = RelaxNGToXSDConverter()

    if xsd_path is None:
        base_name = os.path.splitext(relaxng_path)[0]
        xsd_path = f"{base_name}.xsd"

    xsd_content = converter.convert(relaxng_path, xsd_path)
    return xsd_path


# Alternative: Modify your script to support RelaxNG directly
def create_schema_validator(schema_path):
    """
    Create a validator that works with both XSD and RelaxNG schemas
    """
    ext = os.path.splitext(schema_path)[1].lower()

    if ext == '.xsd':
        # Use your existing XSD logic
        schema_doc = etree.parse(schema_path)
        return etree.XMLSchema(schema_doc)
    elif ext in ['.rng', '.rnc']:
        # Use RelaxNG validator
        schema_doc = etree.parse(schema_path)
        return etree.RelaxNG(schema_doc)
    else:
        raise ValueError(f"Unsupported schema format: {ext}")

if __name__ == '__main__':
    import os
    converter = RelaxNGToXSDConverter()
    rng_dir = os.path.join("examples", "rng")
    for file in os.listdir(rng_dir):
        if file.endswith('.rng'):
            print(file)
            file_path = os.path.join(rng_dir, file)
            xsd_content = converter.convert(file_path)
            print(xsd_content)
