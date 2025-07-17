from lxml import etree
import os
from collections import defaultdict


class RelaxNGToXSDConverter:
    def __init__(self):
        self.xsd_ns = "http://www.w3.org/2001/XMLSchema"
        self.rng_ns = "http://relaxng.org/ns/structure/1.0"
        self.type_counter = 0
        self.named_patterns = {}

    def convert(self, relaxng_input, output_file=None):
        """
        Convert RelaxNG schema to XSD

        Args:
            relaxng_input: Path to RelaxNG file or RelaxNG XML string
            output_file: Optional output file path

        Returns:
            XSD as string
        """
        # Parse RelaxNG
        if os.path.isfile(relaxng_input):
            with open(relaxng_input, 'rb') as f:
                rng_doc = etree.parse(f)
        else:
            rng_doc = etree.fromstring(relaxng_input.encode('utf-8'))

        # Create XSD root
        xsd_root = etree.Element(
            f"{{{self.xsd_ns}}}schema",
            nsmap={'xs': self.xsd_ns}
        )
        xsd_root.set("elementFormDefault", "qualified")

        # First pass: collect named patterns (define elements)
        self._collect_named_patterns(rng_doc)

        # Second pass: convert the schema
        self._convert_grammar(rng_doc, xsd_root)

        # Generate output
        result = etree.tostring(xsd_root, pretty_print=True,
                                xml_declaration=True, encoding='UTF-8')

        if output_file:
            with open(output_file, 'wb') as f:
                f.write(result)

        return result.decode('utf-8')

    def _collect_named_patterns(self, node):
        """Collect all named patterns (define elements) for reference resolution"""
        for define in node.xpath('.//rng:define', namespaces={'rng': self.rng_ns}):
            name = define.get('name')
            if name:
                self.named_patterns[name] = define

    def _convert_grammar(self, rng_node, xsd_parent):
        """Convert RelaxNG grammar to XSD"""
        # Find the start element or root pattern
        start = rng_node.find(f'.//{{{self.rng_ns}}}start')
        if start is not None:
            for child in start:
                self._convert_pattern(child, xsd_parent)
        else:
            # If no explicit start, process all top-level elements
            for child in rng_node:
                if self._get_local_name(child.tag) != 'define':
                    self._convert_pattern(child, xsd_parent)

    def _convert_pattern(self, rng_elem, xsd_parent):
        """Convert a RelaxNG pattern to XSD"""
        tag = self._get_local_name(rng_elem.tag)

        if tag == 'element':
            return self._convert_element(rng_elem, xsd_parent)
        elif tag == 'attribute':
            return self._convert_attribute(rng_elem, xsd_parent)
        elif tag == 'ref':
            return self._convert_ref(rng_elem, xsd_parent)
        elif tag == 'choice':
            return self._convert_choice(rng_elem, xsd_parent)
        elif tag == 'optional':
            return self._convert_optional(rng_elem, xsd_parent)
        elif tag == 'zeroOrMore':
            return self._convert_zero_or_more(rng_elem, xsd_parent)
        elif tag == 'oneOrMore':
            return self._convert_one_or_more(rng_elem, xsd_parent)
        elif tag == 'group':
            return self._convert_group(rng_elem, xsd_parent)
        elif tag == 'interleave':
            # Interleave is approximated as sequence in XSD
            return self._convert_sequence(rng_elem, xsd_parent)
        elif tag == 'text':
            return self._convert_text(rng_elem, xsd_parent)
        elif tag == 'data':
            return self._convert_data(rng_elem, xsd_parent)
        elif tag == 'value':
            return self._convert_value(rng_elem, xsd_parent)
        elif tag == 'empty':
            return None  # Empty pattern
        else:
            # Process children for unknown patterns
            for child in rng_elem:
                self._convert_pattern(child, xsd_parent)

    def _convert_element(self, rng_elem, xsd_parent):
        """Convert RelaxNG element to XSD element"""
        name = rng_elem.get('name')

        if name:
            # Named element
            xsd_elem = etree.SubElement(xsd_parent, f"{{{self.xsd_ns}}}element")
            xsd_elem.set('name', name)

            # Check if element has content
            has_attributes = any(self._get_local_name(child.tag) == 'attribute'
                                 for child in rng_elem)
            has_elements = any(self._get_local_name(child.tag) in
                               ['element', 'ref', 'choice', 'optional', 'zeroOrMore', 'oneOrMore', 'group']
                               for child in rng_elem)
            has_text = any(self._get_local_name(child.tag) in ['text', 'data', 'value']
                           for child in rng_elem)

            if has_attributes or has_elements or (has_text and (has_attributes or has_elements)):
                # Complex type
                complex_type = etree.SubElement(xsd_elem, f"{{{self.xsd_ns}}}complexType")

                if has_text and has_elements:
                    # Mixed content
                    complex_type.set('mixed', 'true')

                # Create sequence for child elements
                if has_elements or len(rng_elem) > 1:
                    sequence = etree.SubElement(complex_type, f"{{{self.xsd_ns}}}sequence")
                    content_parent = sequence
                else:
                    content_parent = complex_type

                # Process children
                for child in rng_elem:
                    child_tag = self._get_local_name(child.tag)
                    if child_tag == 'attribute':
                        self._convert_attribute(child, complex_type)
                    elif child_tag == 'text' and not has_elements:
                        # Simple content with attributes
                        simple_content = etree.SubElement(complex_type, f"{{{self.xsd_ns}}}simpleContent")
                        extension = etree.SubElement(simple_content, f"{{{self.xsd_ns}}}extension")
                        extension.set('base', 'xs:string')
                        # Move attributes to extension
                        for attr in complex_type.findall(f"{{{self.xsd_ns}}}attribute"):
                            complex_type.remove(attr)
                            extension.append(attr)
                    else:
                        self._convert_pattern(child, content_parent)

            elif has_text:
                # Simple type
                xsd_elem.set('type', 'xs:string')

            return xsd_elem
        else:
            # Choice of named elements
            for child in rng_elem:
                self._convert_pattern(child, xsd_parent)

    def _convert_attribute(self, rng_elem, xsd_parent):
        """Convert RelaxNG attribute to XSD attribute"""
        name = rng_elem.get('name')
        if name:
            xsd_attr = etree.SubElement(xsd_parent, f"{{{self.xsd_ns}}}attribute")
            xsd_attr.set('name', name)

            # Determine type from content
            data_elem = rng_elem.find(f'.//{{{self.rng_ns}}}data')
            value_elem = rng_elem.find(f'.//{{{self.rng_ns}}}value')

            if data_elem is not None:
                xsd_type = self._get_xsd_type(data_elem.get('type', 'string'))
                xsd_attr.set('type', xsd_type)
            elif value_elem is not None:
                # Fixed value
                xsd_attr.set('fixed', value_elem.text)
                xsd_attr.set('type', 'xs:string')
            else:
                xsd_attr.set('type', 'xs:string')

            return xsd_attr

    def _convert_ref(self, rng_elem, xsd_parent):
        """Convert RelaxNG ref to corresponding pattern"""
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
        """Convert RelaxNG optional to XSD with minOccurs=0"""
        for child in rng_elem:
            result = self._convert_pattern(child, xsd_parent)
            if result is not None and 'minOccurs' not in result.attrib:
                result.set('minOccurs', '0')

    def _convert_zero_or_more(self, rng_elem, xsd_parent):
        """Convert RelaxNG zeroOrMore to XSD with minOccurs=0 maxOccurs=unbounded"""
        for child in rng_elem:
            result = self._convert_pattern(child, xsd_parent)
            if result is not None:
                result.set('minOccurs', '0')
                result.set('maxOccurs', 'unbounded')

    def _convert_one_or_more(self, rng_elem, xsd_parent):
        """Convert RelaxNG oneOrMore to XSD with maxOccurs=unbounded"""
        for child in rng_elem:
            result = self._convert_pattern(child, xsd_parent)
            if result is not None:
                result.set('maxOccurs', 'unbounded')

    def _convert_group(self, rng_elem, xsd_parent):
        """Convert RelaxNG group to XSD sequence"""
        return self._convert_sequence(rng_elem, xsd_parent)

    def _convert_sequence(self, rng_elem, xsd_parent):
        """Convert to XSD sequence"""
        xsd_sequence = etree.SubElement(xsd_parent, f"{{{self.xsd_ns}}}sequence")
        for child in rng_elem:
            self._convert_pattern(child, xsd_sequence)
        return xsd_sequence

    def _convert_text(self, rng_elem, xsd_parent):
        """Convert RelaxNG text pattern"""
        # Text is typically handled by the parent element
        pass

    def _convert_data(self, rng_elem, xsd_parent):
        """Convert RelaxNG data pattern"""
        # This is typically used within attributes or simple content
        pass

    def _convert_value(self, rng_elem, xsd_parent):
        """Convert RelaxNG value pattern"""
        # This is typically used for fixed values
        pass

    def _get_xsd_type(self, rng_type):
        """Map RelaxNG data types to XSD types"""
        type_mapping = {
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
            'duration': 'xs:duration',
            'anyURI': 'xs:anyURI',
            'ID': 'xs:ID',
            'IDREF': 'xs:IDREF',
            'NMTOKEN': 'xs:NMTOKEN',
        }
        return type_mapping.get(rng_type, 'xs:string')

    def _get_local_name(self, tag):
        """Get local name from tag"""
        if '}' in tag:
            return tag.split('}')[1]
        return tag


# Example usage
def convert_relaxng_to_xsd(relaxng_path, xsd_path=None):
    """
    Convert a RelaxNG schema file to XSD

    Args:
        relaxng_path: Path to the RelaxNG file
        xsd_path: Optional path for the output XSD file

    Returns:
        XSD content as string
    """
    converter = RelaxNGToXSDConverter()
    return converter.convert(relaxng_path, xsd_path)


# Integration function for your existing script
def ensure_xsd_schema(schema_path):
    """
    Ensure the schema is in XSD format, converting from RelaxNG if necessary

    Args:
        schema_path: Path to the schema file

    Returns:
        Path to XSD file (either original or converted)
    """
    import tempfile
    from pathlib import Path

    schema_path = Path(schema_path)

    if schema_path.suffix.lower() == '.xsd':
        return str(schema_path)

    elif schema_path.suffix.lower() in ['.rng', '.rnc']:
        # Convert RelaxNG to XSD
        converter = RelaxNGToXSDConverter()

        # Create temporary XSD file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.xsd', delete=False) as tmp:
            xsd_content = converter.convert(str(schema_path))
            tmp.write(xsd_content)
            return tmp.name

    else:
        raise ValueError(f"Unsupported schema format: {schema_path.suffix}")


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
