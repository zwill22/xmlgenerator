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

            # Check element content
            children = list(rng_elem)

            # Check for simple type with data
            if len(children) == 1 and self._get_local_name(children[0].tag) == 'data':
                # Simple type element
                data_type = self._extract_data_type(children[0])
                xsd_elem.set('type', data_type)
                return xsd_elem

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

                # Handle simple content with attributes
                if has_text and has_attributes and not has_elements:
                    simple_content = etree.SubElement(complex_type, f"{{{self.xsd_ns}}}simpleContent")
                    extension = etree.SubElement(simple_content, f"{{{self.xsd_ns}}}extension")

                    # Find the data type
                    data_elem = rng_elem.find(f'.//{{{self.rng_ns}}}data')
                    if data_elem is not None:
                        base_type = self._extract_data_type(data_elem)
                    else:
                        base_type = 'xs:string'
                    extension.set('base', base_type)

                    # Add attributes to extension
                    for child in rng_elem:
                        if self._get_local_name(child.tag) == 'attribute':
                            self._convert_attribute(child, extension)
                else:
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
                        elif child_tag != 'text' or has_elements:
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

            # Look for data type information
            for child in rng_elem:
                child_tag = self._get_local_name(child.tag)

                if child_tag == 'data':
                    xsd_type = self._extract_data_type(child)
                    xsd_attr.set('type', xsd_type)

                    # Handle patterns/restrictions
                    param_elems = child.findall(f'.//{{{self.rng_ns}}}param')
                    if param_elems:
                        self._add_restrictions(xsd_attr, child, xsd_type)
                    break

                elif child_tag == 'value':
                    # Fixed value
                    xsd_attr.set('fixed', child.text or '')
                    xsd_attr.set('type', 'xs:string')
                    break

                elif child_tag == 'choice':
                    # Enumeration
                    self._convert_attribute_choice(child, xsd_attr)
                    break

                elif child_tag == 'text':
                    xsd_attr.set('type', 'xs:string')
                    break
            else:
                # Default to string if no type specified
                xsd_attr.set('type', 'xs:string')

            return xsd_attr

    def _extract_data_type(self, data_elem):
        """Extract the data type from a RelaxNG data element"""
        # Get the type attribute
        rng_type = data_elem.get('type')

        if not rng_type:
            # Check for datatypeLibrary
            datatype_lib = data_elem.get('datatypeLibrary')
            if datatype_lib and 'XMLSchema' in datatype_lib:
                # Sometimes the type is in a different format
                return 'xs:string'
            return 'xs:string'

        # Handle different type formats
        if rng_type.startswith('xsd:'):
            # Already in XSD format
            return rng_type.replace('xsd:', 'xs:')
        elif ':' in rng_type:
            # Has namespace prefix
            prefix, local_type = rng_type.split(':', 1)
            return f'xs:{local_type}'
        else:
            # Map to XSD type
            return self._get_xsd_type(rng_type)

    def _convert_attribute_choice(self, choice_elem, xsd_attr):
        """Convert attribute choice to enumeration"""
        # Create a simple type with restriction
        simple_type = etree.SubElement(xsd_attr.getparent(), f"{{{self.xsd_ns}}}simpleType")
        restriction = etree.SubElement(simple_type, f"{{{self.xsd_ns}}}restriction")
        restriction.set('base', 'xs:string')

        # Add enumeration values
        for value_elem in choice_elem.findall(f'.//{{{self.rng_ns}}}value'):
            enum = etree.SubElement(restriction, f"{{{self.xsd_ns}}}enumeration")
            enum.set('value', value_elem.text or '')

        # Remove type attribute and link to simple type
        if 'type' in xsd_attr.attrib:
            del xsd_attr.attrib['type']

        # Generate unique name for the simple type
        type_name = f"{xsd_attr.get('name')}Type"
        simple_type.set('name', type_name)
        xsd_attr.set('type', type_name)

        # Move simple type before the attribute
        parent = xsd_attr.getparent()
        parent.remove(simple_type)
        parent.insert(parent.index(xsd_attr), simple_type)

    def _add_restrictions(self, xsd_elem, data_elem, base_type):
        """Add restrictions from RelaxNG param elements"""
        params = {}
        for param in data_elem.findall(f'.//{{{self.rng_ns}}}param'):
            name = param.get('name')
            if name:
                params[name] = param.text

        if params:
            # Create inline simple type with restrictions
            simple_type = etree.SubElement(xsd_elem.getparent(), f"{{{self.xsd_ns}}}simpleType")
            restriction = etree.SubElement(simple_type, f"{{{self.xsd_ns}}}restriction")
            restriction.set('base', base_type)

            # Map RelaxNG params to XSD facets
            param_mapping = {
                'minLength': 'minLength',
                'maxLength': 'maxLength',
                'pattern': 'pattern',
                'minInclusive': 'minInclusive',
                'maxInclusive': 'maxInclusive',
                'minExclusive': 'minExclusive',
                'maxExclusive': 'maxExclusive',
                'totalDigits': 'totalDigits',
                'fractionDigits': 'fractionDigits'
            }

            for param_name, param_value in params.items():
                if param_name in param_mapping:
                    facet = etree.SubElement(restriction, f"{{{self.xsd_ns}}}{param_mapping[param_name]}")
                    facet.set('value', param_value)

            # Remove type attribute and use inline simple type
            if 'type' in xsd_elem.attrib:
                del xsd_elem.attrib['type']

    def _get_xsd_type(self, rng_type):
        """Map RelaxNG data types to XSD types"""
        type_mapping = {
            # String types
            'string': 'xs:string',
            'token': 'xs:token',
            'normalizedString': 'xs:normalizedString',
            'language': 'xs:language',
            'Name': 'xs:Name',
            'NCName': 'xs:NCName',
            'NMTOKEN': 'xs:NMTOKEN',
            'NMTOKENS': 'xs:NMTOKENS',

            # Numeric types
            'integer': 'xs:integer',
            'positiveInteger': 'xs:positiveInteger',
            'negativeInteger': 'xs:negativeInteger',
            'nonNegativeInteger': 'xs:nonNegativeInteger',
            'nonPositiveInteger': 'xs:nonPositiveInteger',
            'int': 'xs:int',
            'unsignedInt': 'xs:unsignedInt',
            'long': 'xs:long',
            'unsignedLong': 'xs:unsignedLong',
            'short': 'xs:short',
            'unsignedShort': 'xs:unsignedShort',
            'byte': 'xs:byte',
            'unsignedByte': 'xs:unsignedByte',
            'decimal': 'xs:decimal',
            'float': 'xs:float',
            'double': 'xs:double',

            # Boolean
            'boolean': 'xs:boolean',

            # Date/Time types
            'date': 'xs:date',
            'dateTime': 'xs:dateTime',
            'time': 'xs:time',
            'duration': 'xs:duration',
            'gYear': 'xs:gYear',
            'gYearMonth': 'xs:gYearMonth',
            'gMonth': 'xs:gMonth',
            'gMonthDay': 'xs:gMonthDay',
            'gDay': 'xs:gDay',

            # Other types
            'anyURI': 'xs:anyURI',
            'base64Binary': 'xs:base64Binary',
            'hexBinary': 'xs:hexBinary',
            'QName': 'xs:QName',
            'ID': 'xs:ID',
            'IDREF': 'xs:IDREF',
            'IDREFS': 'xs:IDREFS',
            'ENTITY': 'xs:ENTITY',
            'ENTITIES': 'xs:ENTITIES',
        }
        return type_mapping.get(rng_type, 'xs:string')

    def _get_local_name(self, tag):
        """Get local name from tag"""
        if '}' in tag:
            return tag.split('}')[1]
        return tag

    # ... (rest of the methods remain the same)
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
