import xml.etree.ElementTree as ET
from lxml import etree
from faker import Faker
import random
import re
from datetime import datetime, timedelta
from typing import Dict, Any, Optional, List, Set, Tuple
import os


class CircularReferenceError(Exception):
    """Raised when a circular reference is detected in the schema."""
    pass


class XMLGenerator:
    def __init__(self, schema_path: str):
        self.schema_path = schema_path
        self.fake = Faker()

        # First validate that the schema file exists
        if not os.path.exists(schema_path):
            raise FileNotFoundError(f"Schema file not found: {schema_path}")

        # Parse the schema document
        try:
            self.schema_doc = etree.parse(schema_path)
        except etree.XMLSyntaxError as e:
            raise ValueError(f"Invalid XML in schema file: {e}")

        # Validate that this is actually an XSD schema
        self._validate_schema()

        # Create XMLSchema object for validation
        try:
            self.schema = etree.XMLSchema(self.schema_doc)
        except etree.XMLSchemaParseError as e:
            raise ValueError(f"Invalid XSD schema: {e}")

        self.target_namespace = self.schema_doc.getroot().get('targetNamespace', '')
        self.elements = {}
        self.complex_types = {}
        self.simple_types = {}
        self.referenced_elements = set()
        self._parse_schema()

        # Check for circular references after parsing
        self._check_circular_references()

    def _check_circular_references(self):
        """Check for circular references in the schema."""
        # Check each global element
        for element_name, element_def in self.elements.items():
            visited = set()
            path = []
            if self._has_circular_reference(element_def, visited, path, element_name):
                cycle_path = " -> ".join(path)
                raise CircularReferenceError(
                    f"Circular reference detected in schema starting from element '{element_name}': {cycle_path}"
                )

        # Check each complex type
        for type_name, type_def in self.complex_types.items():
            visited = set()
            path = []
            if self._has_circular_reference_in_type(type_def, visited, path, f"complexType:{type_name}"):
                cycle_path = " -> ".join(path)
                raise CircularReferenceError(
                    f"Circular reference detected in complex type '{type_name}': {cycle_path}"
                )

    def _has_circular_reference(self, element_def, visited: Set[str], path: List[str], current_name: str) -> bool:
        """Check if an element definition has circular references."""
        if current_name in visited:
            # Found a cycle - add current name to path to show the cycle
            path.append(current_name)
            return True

        visited.add(current_name)
        path.append(current_name)

        # Get namespace info
        root = self.schema_doc.getroot()
        ns_map = root.nsmap
        xsd_namespace = "http://www.w3.org/2001/XMLSchema"

        xsd_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == xsd_namespace:
                xsd_prefix = prefix
                break

        if xsd_prefix:
            ns = {xsd_prefix: xsd_namespace}
            prefix = f"{xsd_prefix}:"
        else:
            ns = {}
            prefix = ""

        # Check element type
        type_attr = element_def.get('type', '')
        if type_attr and type_attr in self.complex_types:
            # Check the complex type
            if self._has_circular_reference_in_type(self.complex_types[type_attr], visited, path, f"type:{type_attr}"):
                return True

        # Check inline complex type
        ct_xpath = f'./{prefix}complexType'
        if ns:
            complex_types = element_def.xpath(ct_xpath, namespaces=ns)
        else:
            complex_types = element_def.xpath(ct_xpath)

        for complex_type in complex_types:
            if self._has_circular_reference_in_type(complex_type, visited, path,
                                                    f"inline-complexType-in-{current_name}"):
                return True

        # Check element references
        ref = element_def.get('ref')
        if ref:
            if ref in self.elements:
                if self._has_circular_reference(self.elements[ref], visited, path, ref):
                    return True

        path.pop()
        visited.remove(current_name)
        return False

    def _has_circular_reference_in_type(self, type_def, visited: Set[str], path: List[str], type_id: str) -> bool:
        """Check if a type definition has circular references."""
        if type_id in visited:
            path.append(type_id)
            return True

        visited.add(type_id)
        path.append(type_id)

        # Get namespace info
        root = self.schema_doc.getroot()
        ns_map = root.nsmap
        xsd_namespace = "http://www.w3.org/2001/XMLSchema"

        xsd_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == xsd_namespace:
                xsd_prefix = prefix
                break

        if xsd_prefix:
            ns = {xsd_prefix: xsd_namespace}
            prefix = f"{xsd_prefix}:"
        else:
            ns = {}
            prefix = ""

        # Check all child elements in sequences, choices, and all groups
        element_containers = [
            f'.//{prefix}sequence/{prefix}element',
            f'.//{prefix}choice/{prefix}element',
            f'.//{prefix}all/{prefix}element'
        ]

        for container_xpath in element_containers:
            if ns:
                elements = type_def.xpath(container_xpath, namespaces=ns)
            else:
                elements = type_def.xpath(container_xpath)

            for elem in elements:
                # Check element references
                ref = elem.get('ref')
                if ref:
                    if ref in self.elements:
                        if self._has_circular_reference(self.elements[ref], visited, path, ref):
                            return True
                else:
                    # Check named elements with types
                    elem_name = elem.get('name', 'unnamed')
                    elem_type = elem.get('type', '')

                    if elem_type and elem_type in self.complex_types:
                        if self._has_circular_reference_in_type(self.complex_types[elem_type], visited, path,
                                                                f"type:{elem_type}"):
                            return True

                    # Check inline complex types
                    ct_xpath = f'./{prefix}complexType'
                    if ns:
                        inline_types = elem.xpath(ct_xpath, namespaces=ns)
                    else:
                        inline_types = elem.xpath(ct_xpath)

                    for inline_type in inline_types:
                        if self._has_circular_reference_in_type(inline_type, visited, path,
                                                                f"inline-complexType-in-{elem_name}"):
                            return True

        path.pop()
        visited.remove(type_id)
        return False

    def _validate_schema(self):
        """Validate that the input file is a valid XSD schema."""
        # XSD for XSD (meta-schema)
        xsd_schema_url = "http://www.w3.org/2001/XMLSchema.xsd"

        # Try to load the XSD meta-schema
        try:
            # First try to use the cached version if available
            xsd_schema_doc = None

            # Try to parse from URL (this might fail if offline)
            try:
                xsd_schema_doc = etree.parse(xsd_schema_url)
            except:
                # If online fetch fails, use a minimal validation
                pass

            if xsd_schema_doc:
                xsd_schema = etree.XMLSchema(xsd_schema_doc)
                if not xsd_schema.validate(self.schema_doc):
                    errors = xsd_schema.error_log
                    raise ValueError(f"Invalid XSD schema structure: {errors}")
            else:
                # Fallback: perform basic structural validation
                self._basic_schema_validation()

        except etree.XMLSchemaParseError as e:
            raise ValueError(f"Schema validation failed: {e}")

    def _basic_schema_validation(self):
        """Perform basic validation to ensure this is an XSD schema."""
        root = self.schema_doc.getroot()

        # Check that the root element is xs:schema or xsd:schema
        if not (root.tag.endswith('}schema') or root.tag == 'schema'):
            raise ValueError("Root element must be 'schema'")

        # Check for XSD namespace
        namespaces = root.nsmap
        xsd_namespace = "http://www.w3.org/2001/XMLSchema"

        if not any(ns == xsd_namespace for ns in namespaces.values()):
            # Check if it's using the namespace without prefix
            if root.tag != f"{{{xsd_namespace}}}schema":
                raise ValueError("Schema must use XML Schema namespace")

        # Check for at least one element, type, or attribute definition
        ns = {'xs': xsd_namespace, 'xsd': xsd_namespace}

        has_definitions = False
        for prefix in ['xs', 'xsd', '']:
            if prefix:
                xpath_prefix = f"{prefix}:"
            else:
                xpath_prefix = ""

            # Try different namespace prefixes
            try:
                elements = root.xpath(f'//{xpath_prefix}element', namespaces=ns)
                types = root.xpath(f'//{xpath_prefix}complexType | //{xpath_prefix}simpleType', namespaces=ns)
                attributes = root.xpath(f'//{xpath_prefix}attribute', namespaces=ns)

                if elements or types or attributes:
                    has_definitions = True
                    break
            except:
                continue

        if not has_definitions:
            # Try without namespace prefix
            elements = root.xpath('//element')
            types = root.xpath('//complexType | //simpleType')
            attributes = root.xpath('//attribute')

            if not (elements or types or attributes):
                raise ValueError("Schema must contain at least one element, type, or attribute definition")

    def _parse_schema(self):
        """Parse the XSD schema and extract element, complex type, and simple type definitions."""
        root = self.schema_doc.getroot()

        # Determine the namespace prefix used in the schema
        ns_map = root.nsmap
        xsd_namespace = "http://www.w3.org/2001/XMLSchema"

        # Find the prefix for XSD namespace
        xsd_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == xsd_namespace:
                xsd_prefix = prefix
                break

        # Build namespace dict for XPath
        if xsd_prefix:
            ns = {xsd_prefix: xsd_namespace}
            prefix = f"{xsd_prefix}:"
        else:
            # Schema might be using default namespace
            ns = {'xs': xsd_namespace}
            prefix = "xs:"

            # Check if it's using XSD namespace as default
            if root.tag == f"{{{xsd_namespace}}}schema":
                # Use empty prefix for default namespace
                prefix = ""
                ns = {}

        # First, find all referenced elements
        if ns:
            ref_xpath = f'//{prefix}element[@ref]'
            for ref in root.xpath(ref_xpath, namespaces=ns):
                ref_name = ref.get('ref')
                if ref_name:
                    self.referenced_elements.add(ref_name)
        else:
            # Try without namespace
            for ref in root.xpath('//element[@ref]'):
                ref_name = ref.get('ref')
                if ref_name:
                    self.referenced_elements.add(ref_name)

        # Extract elements
        if ns:
            elem_xpath = f'//{prefix}element[@name]'
            for elem in root.xpath(elem_xpath, namespaces=ns):
                name = elem.get('name')
                if name:
                    self.elements[name] = elem
        else:
            for elem in root.xpath('//element[@name]'):
                name = elem.get('name')
                if name:
                    self.elements[name] = elem

        # Extract complex types
        if ns:
            ct_xpath = f'//{prefix}complexType[@name]'
            for ct in root.xpath(ct_xpath, namespaces=ns):
                name = ct.get('name')
                if name:
                    self.complex_types[name] = ct
        else:
            for ct in root.xpath('//complexType[@name]'):
                name = ct.get('name')
                if name:
                    self.complex_types[name] = ct

        # Extract simple types
        if ns:
            st_xpath = f'//{prefix}simpleType[@name]'
            for st in root.xpath(st_xpath, namespaces=ns):
                name = st.get('name')
                if name:
                    self.simple_types[name] = st
        else:
            for st in root.xpath('//simpleType[@name]'):
                name = st.get('name')
                if name:
                    self.simple_types[name] = st

        # Validate that we found something
        if not (self.elements or self.complex_types or self.simple_types):
            raise ValueError("No elements, complex types, or simple types found in schema")

    def _find_root_element(self) -> str:
        """Find the most appropriate root element."""
        # Root element is typically one that:
        # 1. Is not referenced by other elements
        # 2. Is a global element (direct child of schema)
        # 3. Preferably has a complex type

        root = self.schema_doc.getroot()
        xsd_namespace = "http://www.w3.org/2001/XMLSchema"

        # Find namespace prefix
        ns_map = root.nsmap
        xsd_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == xsd_namespace:
                xsd_prefix = prefix
                break

        if xsd_prefix:
            ns = {xsd_prefix: xsd_namespace}
            elem_xpath = f'./{xsd_prefix}:element[@name]'
        else:
            ns = {}
            elem_xpath = './element[@name]'

        # Get all global elements (direct children of schema)
        if ns:
            global_elements = root.xpath(elem_xpath, namespaces=ns)
        else:
            global_elements = root.xpath(elem_xpath)

        potential_roots = []

        for elem in global_elements:
            name = elem.get('name')
            if name and name not in self.referenced_elements:
                # Check if it has a complex type
                type_attr = elem.get('type', '')

                # Check for inline complex type
                if xsd_prefix:
                    has_inline_complex = elem.xpath(f'./{xsd_prefix}:complexType', namespaces=ns)
                else:
                    has_inline_complex = elem.xpath('./complexType')

                has_complex_type = (
                        type_attr in self.complex_types or
                        has_inline_complex or
                        (type_attr and not type_attr.startswith('xs:'))
                )

                if has_complex_type:
                    potential_roots.insert(0, name)  # Prefer complex types
                else:
                    potential_roots.append(name)

        # Return the first potential root
        if potential_roots:
            return potential_roots[0]

        # If no global elements found, fall back to any element not referenced
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
                # Handle pattern restrictions
                pattern = restrictions['pattern']
                if pattern == '[0-9]{6}':
                    return ''.join([str(random.randint(0, 9)) for _ in range(6)])
                elif r'\d' in pattern:
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

        # Determine namespace setup
        root = self.schema_doc.getroot()
        ns_map = root.nsmap
        xsd_namespace = "http://www.w3.org/2001/XMLSchema"

        xsd_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == xsd_namespace:
                xsd_prefix = prefix
                break

        if xsd_prefix:
            ns = {xsd_prefix: xsd_namespace}
            prefix = f"{xsd_prefix}:"
        else:
            ns = {}
            prefix = ""

        # Look for restrictions
        restriction_xpath = f'.//{prefix}restriction'
        if ns:
            restrictions_elem = element.xpath(restriction_xpath, namespaces=ns)
        else:
            restrictions_elem = element.xpath(restriction_xpath)

        for restriction in restrictions_elem:
            # Enumerations
            enum_xpath = f'.//{prefix}enumeration/@value'
            if ns:
                enums = restriction.xpath(enum_xpath, namespaces=ns)
            else:
                enums = restriction.xpath(enum_xpath)
            if enums:
                restrictions['enumeration'] = enums

            # Pattern
            pattern_xpath = f'.//{prefix}pattern/@value'
            if ns:
                pattern = restriction.xpath(pattern_xpath, namespaces=ns)
            else:
                pattern = restriction.xpath(pattern_xpath)
            if pattern:
                restrictions['pattern'] = pattern[0]

            # Length restrictions
            for attr in ['minLength', 'maxLength', 'length']:
                attr_xpath = f'.//{prefix}{attr}/@value'
                if ns:
                    value = restriction.xpath(attr_xpath, namespaces=ns)
                else:
                    value = restriction.xpath(attr_xpath)
                if value:
                    restrictions[attr] = int(value[0])

            # Numeric restrictions
            for attr in ['minInclusive', 'maxInclusive', 'minExclusive', 'maxExclusive']:
                attr_xpath = f'.//{prefix}{attr}/@value'
                if ns:
                    value = restriction.xpath(attr_xpath, namespaces=ns)
                else:
                    value = restriction.xpath(attr_xpath)
                if value:
                    restrictions[attr] = value[0]

        return restrictions

    def _generate_element(self, element_def, parent_element: ET.Element, depth: int = 0):
        """Generate XML element based on schema definition."""
        # This should never be reached now due to circular reference checking
        if depth > 10:
            raise CircularReferenceError("Maximum recursion depth exceeded - possible undetected circular reference")

        # Setup namespace handling
        root = self.schema_doc.getroot()
        ns_map = root.nsmap
        xsd_namespace = "http://www.w3.org/2001/XMLSchema"

        xsd_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == xsd_namespace:
                xsd_prefix = prefix
                break

        if xsd_prefix:
            ns = {xsd_prefix: xsd_namespace}
            prefix = f"{xsd_prefix}:"
        else:
            ns = {}
            prefix = ""

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
            max_occurs = random.randint(2, 4)  # Generate 2-4 elements for unbounded
        else:
            max_occurs = int(max_occurs)

        # Ensure we generate at least minOccurs and possibly more
        if max_occurs > min_occurs:
            occurs = random.randint(min_occurs, max_occurs)
        else:
            occurs = min_occurs

        for _ in range(occurs):
            elem = ET.SubElement(parent_element, name)

            # Handle element content
            has_content = False
            if type_attr:
                if type_attr.startswith('xs:') or type_attr in ['string', 'integer', 'double', 'float', 'boolean',
                                                                'date', 'dateTime', 'time', 'positiveInteger']:
                    # Simple XSD type
                    restrictions = self._extract_restrictions(element_def)
                    elem.text = self._get_fake_data_for_type(type_attr, restrictions)
                    has_content = True
                elif type_attr in self.complex_types:
                    # Named complex type
                    self._generate_complex_type(self.complex_types[type_attr], elem, depth + 1)
                    has_content = True
                elif type_attr in self.simple_types:
                    # Named simple type
                    simple_type_def = self.simple_types[type_attr]
                    restrictions = self._extract_restrictions(simple_type_def)
                    base_xpath = f'.//{prefix}restriction/@base'
                    if ns:
                        base_type = simple_type_def.xpath(base_xpath, namespaces=ns)
                    else:
                        base_type = simple_type_def.xpath(base_xpath)
                    if base_type:
                        elem.text = self._get_fake_data_for_type(base_type[0], restrictions)
                    else:
                        elem.text = self._get_fake_data_for_type('xs:string', restrictions)
                    has_content = True
                else:
                    # Unknown type, treat as string
                    elem.text = self.fake.word()
                    has_content = True
            else:
                # Inline type definition
                ct_xpath = f'./{prefix}complexType'
                st_xpath = f'./{prefix}simpleType'

                if ns:
                    complex_type = element_def.xpath(ct_xpath, namespaces=ns)
                    simple_type = element_def.xpath(st_xpath, namespaces=ns)
                else:
                    complex_type = element_def.xpath(ct_xpath)
                    simple_type = element_def.xpath(st_xpath)

                if complex_type:
                    self._generate_complex_type(complex_type[0], elem, depth + 1)
                    has_content = True
                elif simple_type:
                    restrictions = self._extract_restrictions(simple_type[0])
                    base_xpath = f'.//{prefix}restriction/@base'
                    if ns:
                        base_type = simple_type[0].xpath(base_xpath, namespaces=ns)
                    else:
                        base_type = simple_type[0].xpath(base_xpath)
                    if base_type:
                        elem.text = self._get_fake_data_for_type(base_type[0], restrictions)
                    else:
                        elem.text = self._get_fake_data_for_type('xs:string', restrictions)
                    has_content = True

            # Handle attributes after content generation
            attr_xpath = f'.//{prefix}attribute'
            if ns:
                attributes = element_def.xpath(attr_xpath, namespaces=ns)
            else:
                attributes = element_def.xpath(attr_xpath)

            for attr in attributes:
                attr_name = attr.get('name')
                attr_type = attr.get('type', 'xs:string')
                attr_use = attr.get('use', 'optional')

                if attr_use == 'required' or random.choice([True, False]):
                    restrictions = self._extract_restrictions(attr)
                    if attr_type in self.simple_types:
                        # Handle named simple types for attributes
                        simple_type_def = self.simple_types[attr_type]
                        restrictions.update(self._extract_restrictions(simple_type_def))
                        base_xpath = f'.//{prefix}restriction/@base'
                        if ns:
                            base_type = simple_type_def.xpath(base_xpath, namespaces=ns)
                        else:
                            base_type = simple_type_def.xpath(base_xpath)
                        if base_type:
                            attr_value = self._get_fake_data_for_type(base_type[0], restrictions)
                        else:
                            attr_value = self._get_fake_data_for_type('xs:string', restrictions)
                    else:
                        attr_value = self._get_fake_data_for_type(attr_type, restrictions)
                    elem.set(attr_name, attr_value)

            # Ensure empty elements have explicit closing tags
            if not has_content and not elem.text and len(elem) == 0:
                elem.text = ""

    def _generate_complex_type(self, complex_type_def, parent_element: ET.Element, depth: int = 0):
        """Generate content for a complex type."""
        if depth > 10:
            raise CircularReferenceError("Maximum recursion depth exceeded - possible undetected circular reference")

        # Setup namespace handling
        root = self.schema_doc.getroot()
        ns_map = root.nsmap
        xsd_namespace = "http://www.w3.org/2001/XMLSchema"

        xsd_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == xsd_namespace:
                xsd_prefix = prefix
                break

        if xsd_prefix:
            ns = {xsd_prefix: xsd_namespace}
            prefix = f"{xsd_prefix}:"
        else:
            ns = {}
            prefix = ""

        # Handle attributes at complex type level first
        attr_xpath = f'./{prefix}attribute'
        if ns:
            attributes = complex_type_def.xpath(attr_xpath, namespaces=ns)
        else:
            attributes = complex_type_def.xpath(attr_xpath)

        for attr in attributes:
            attr_name = attr.get('name')
            attr_type = attr.get('type', 'xs:string')
            attr_use = attr.get('use', 'optional')

            if attr_use == 'required' or random.choice([True, False]):
                restrictions = self._extract_restrictions(attr)
                if attr_type in self.simple_types:
                    # Handle named simple types for attributes
                    simple_type_def = self.simple_types[attr_type]
                    restrictions.update(self._extract_restrictions(simple_type_def))
                    base_xpath = f'.//{prefix}restriction/@base'
                    if ns:
                        base_type = simple_type_def.xpath(base_xpath, namespaces=ns)
                    else:
                        base_type = simple_type_def.xpath(base_xpath)
                    if base_type:
                        attr_value = self._get_fake_data_for_type(base_type[0], restrictions)
                    else:
                        attr_value = self._get_fake_data_for_type('xs:string', restrictions)
                else:
                    attr_value = self._get_fake_data_for_type(attr_type, restrictions)
                parent_element.set(attr_name, attr_value)

        # Handle sequences
        seq_xpath = f'.//{prefix}sequence'
        if ns:
            sequences = complex_type_def.xpath(seq_xpath, namespaces=ns)
        else:
            sequences = complex_type_def.xpath(seq_xpath)

        for sequence in sequences:
            elem_xpath = f'./{prefix}element'
            if ns:
                child_elements = sequence.xpath(elem_xpath, namespaces=ns)
            else:
                child_elements = sequence.xpath(elem_xpath)
            for child_elem in child_elements:
                self._generate_element(child_elem, parent_element, depth + 1)

        # Handle choices
        choice_xpath = f'.//{prefix}choice'
        if ns:
            choices = complex_type_def.xpath(choice_xpath, namespaces=ns)
        else:
            choices = complex_type_def.xpath(choice_xpath)

        for choice in choices:
            elem_xpath = f'./{prefix}element'
            if ns:
                child_elements = choice.xpath(elem_xpath, namespaces=ns)
            else:
                child_elements = choice.xpath(elem_xpath)
            if child_elements:
                chosen_elem = random.choice(child_elements)
                self._generate_element(chosen_elem, parent_element, depth + 1)

        # Handle all
        all_xpath = f'.//{prefix}all'
        if ns:
            all_groups = complex_type_def.xpath(all_xpath, namespaces=ns)
        else:
            all_groups = complex_type_def.xpath(all_xpath)

        for all_group in all_groups:
            elem_xpath = f'./{prefix}element'
            if ns:
                child_elements = all_group.xpath(elem_xpath, namespaces=ns)
            else:
                child_elements = all_group.xpath(elem_xpath)
            for child_elem in child_elements:
                self._generate_element(child_elem, parent_element, depth + 1)

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
        elif type_attr and type_attr in self.simple_types:
            # Handle simple type root element
            simple_type_def = self.simple_types[type_attr]
            restrictions = self._extract_restrictions(simple_type_def)

            # Setup namespace for XPath
            root_schema = self.schema_doc.getroot()
            ns_map = root_schema.nsmap
            xsd_namespace = "http://www.w3.org/2001/XMLSchema"

            xsd_prefix = None
            for prefix, namespace in ns_map.items():
                if namespace == xsd_namespace:
                    xsd_prefix = prefix
                    break

            if xsd_prefix:
                ns = {xsd_prefix: xsd_namespace}
                base_xpath = f'.//{xsd_prefix}:restriction/@base'
                base_type = simple_type_def.xpath(base_xpath, namespaces=ns)
            else:
                base_xpath = './/restriction/@base'
                base_type = simple_type_def.xpath(base_xpath)

            if base_type:
                root.text = self._get_fake_data_for_type(base_type[0], restrictions)
            else:
                root.text = self._get_fake_data_for_type('xs:string', restrictions)
        else:
            # Handle inline complex type or simple content
            root_schema = self.schema_doc.getroot()
            ns_map = root_schema.nsmap
            xsd_namespace = "http://www.w3.org/2001/XMLSchema"

            xsd_prefix = None
            for prefix, namespace in ns_map.items():
                if namespace == xsd_namespace:
                    xsd_prefix = prefix
                    break

            if xsd_prefix:
                ns = {xsd_prefix: xsd_namespace}
                ct_xpath = f'./{xsd_prefix}:complexType'
                st_xpath = f'./{xsd_prefix}:simpleType'
                complex_type = root_def.xpath(ct_xpath, namespaces=ns)
                simple_type = root_def.xpath(st_xpath, namespaces=ns)
            else:
                complex_type = root_def.xpath('./complexType')
                simple_type = root_def.xpath('./simpleType')

            if complex_type:
                self._generate_complex_type(complex_type[0], root)
            elif simple_type:
                restrictions = self._extract_restrictions(simple_type[0])
                if xsd_prefix:
                    base_xpath = f'.//{xsd_prefix}:restriction/@base'
                    base_type = simple_type[0].xpath(base_xpath, namespaces=ns)
                else:
                    base_xpath = './/restriction/@base'
                    base_type = simple_type[0].xpath(base_xpath)
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
            tree.write(output_path, encoding='utf-8', xml_declaration=True)
            return output_path
        else:
            # Return XML string with declaration
            xml_str = ET.tostring(root, encoding='unicode')
            return '<?xml version="1.0" encoding="UTF-8"?>\n' + xml_str


def generate_xml_from_schema(schema_path: str, root_element: str = None, output_path: str = None) -> str:
    """
    Generate XML file with fake data based on XSD schema.

    Args:
        schema_path: Path to the XSD schema file
        root_element: Name of the root element (optional, will use first global element if not specified)
        output_path: Path where to save the generated XML file (optional)

    Returns:
        Generated XML as string or path to saved file

    Raises:
        FileNotFoundError: If schema file doesn't exist
        ValueError: If schema is invalid or malformed
        CircularReferenceError: If schema contains circular references
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
            try:
                result = generate_xml_from_schema(schema)
                print(result)
            except CircularReferenceError as e:
                print(e)
