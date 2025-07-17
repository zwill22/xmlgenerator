from abc import ABC, abstractmethod
from typing import Dict, Any, Optional, List, Set
import xml.etree.ElementTree as ET
from lxml import etree
from faker import Faker
import random
import os


class SchemaParser(ABC):
    """Abstract base class for schema parsers."""

    @abstractmethod
    def parse_schema(self, schema_path: str) -> None:
        """Parse the schema file and extract definitions."""
        pass

    @abstractmethod
    def validate_schema(self) -> None:
        """Validate that the schema is well-formed."""
        pass

    @abstractmethod
    def find_root_element(self) -> str:
        """Find the most appropriate root element."""
        pass

    @abstractmethod
    def generate_element(self, element_def, parent_element: ET.Element, depth: int = 0) -> None:
        """Generate XML element based on schema definition."""
        pass

    @abstractmethod
    def check_circular_references(self) -> None:
        """Check for circular references in the schema."""
        pass


class XMLGenerator:
    """Main XML generator that works with different schema formats."""

    def __init__(self, schema_path: str, schema_format: str = 'auto'):
        self.schema_path = schema_path
        self.fake = Faker()

        # Detect schema format if not specified
        if schema_format == 'auto':
            schema_format = self._detect_schema_format(schema_path)

        # Create appropriate parser
        self.parser = self._create_parser(schema_format)
        self.parser.parse_schema(schema_path)
        self.parser.validate_schema()
        self.parser.check_circular_references()

    def _detect_schema_format(self, schema_path: str) -> str:
        """Auto-detect schema format from file content."""
        try:
            doc = etree.parse(schema_path)
            root = doc.getroot()

            # Check for XSD
            if (root.tag.endswith('}schema') or root.tag == 'schema') and \
                    'http://www.w3.org/2001/XMLSchema' in (root.nsmap.values() if root.nsmap else []):
                return 'xsd'

            # Check for Relax NG
            if (root.tag.endswith('}grammar') or root.tag == 'grammar') and \
                    'http://relaxng.org/ns/structure/1.0' in (root.nsmap.values() if root.nsmap else []):
                return 'relaxng'

            # Check for Relax NG with different root elements
            if root.tag.endswith('}element') and \
                    'http://relaxng.org/ns/structure/1.0' in (root.nsmap.values() if root.nsmap else []):
                return 'relaxng'

        except Exception:
            pass

        # Fallback to file extension
        if schema_path.endswith('.xsd'):
            return 'xsd'
        elif schema_path.endswith('.rng'):
            return 'relaxng'
        elif schema_path.endswith('.rnc'):
            return 'relaxng_compact'

        raise ValueError(f"Could not detect schema format for {schema_path}")

    def _create_parser(self, schema_format: str) -> SchemaParser:
        """Create appropriate parser based on schema format."""
        if schema_format == 'xsd':
            return XSDParser()
        elif schema_format == 'relaxng':
            return RelaxNGParser()
        elif schema_format == 'relaxng_compact':
            return RelaxNGCompactParser()
        else:
            raise ValueError(f"Unsupported schema format: {schema_format}")

    def generate_xml(self, root_element_name: str = None, output_path: str = None) -> str:
        """Generate XML file with fake data."""
        if root_element_name is None:
            root_element_name = self.parser.find_root_element()
            if not root_element_name:
                raise ValueError("No suitable root element found in schema")

        # Create root element
        root = ET.Element(root_element_name)

        # Generate content using parser
        self.parser.generate_element(None, root, 0)  # Parser will handle root generation

        # Format and return
        tree = ET.ElementTree(root)
        ET.indent(tree, space="  ", level=0)

        if output_path:
            tree.write(output_path, encoding='utf-8', xml_declaration=True)
            return output_path
        else:
            xml_str = ET.tostring(root, encoding='unicode')
            return '<?xml version="1.0" encoding="UTF-8"?>\n' + xml_str


class XSDParser(SchemaParser):
    """Parser for XSD schemas - this would be the existing XSD code."""

    def __init__(self):
        self.elements = {}
        self.complex_types = {}
        self.simple_types = {}
        self.referenced_elements = set()
        self.schema_doc = None
        self.fake = Faker()

    # ... (existing XSD implementation would go here)


class RelaxNGParser(SchemaParser):
    """Parser for Relax NG schemas."""

    def __init__(self):
        self.elements = {}
        self.patterns = {}
        self.schema_doc = None
        self.fake = Faker()
        self.root_pattern = None

    def parse_schema(self, schema_path: str) -> None:
        """Parse Relax NG schema."""
        if not os.path.exists(schema_path):
            raise FileNotFoundError(f"Schema file not found: {schema_path}")

        try:
            self.schema_doc = etree.parse(schema_path)
        except etree.XMLSyntaxError as e:
            raise ValueError(f"Invalid XML in schema file: {e}")

        self._extract_patterns()

    def _extract_patterns(self):
        """Extract patterns from Relax NG schema."""
        root = self.schema_doc.getroot()
        rng_ns = "http://relaxng.org/ns/structure/1.0"

        # Find namespace prefix for Relax NG
        ns_map = root.nsmap
        rng_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == rng_ns:
                rng_prefix = prefix
                break

        if rng_prefix:
            ns = {rng_prefix: rng_ns}
            prefix = f"{rng_prefix}:"
        else:
            ns = {}
            prefix = ""

        # Extract named patterns (define elements)
        define_xpath = f'//{prefix}define'
        if ns:
            defines = root.xpath(define_xpath, namespaces=ns)
        else:
            defines = root.xpath(define_xpath)

        for define in defines:
            name = define.get('name')
            if name:
                self.patterns[name] = define

        # Extract start pattern
        start_xpath = f'//{prefix}start'
        if ns:
            starts = root.xpath(start_xpath, namespaces=ns)
        else:
            starts = root.xpath(start_xpath)

        if starts:
            self.root_pattern = starts[0]
        else:
            # If no start element, use the root element directly
            self.root_pattern = root

    def validate_schema(self) -> None:
        """Validate Relax NG schema."""
        try:
            # Use lxml's RelaxNG validator
            relaxng_doc = etree.parse(self.schema_doc)
            relaxng = etree.RelaxNG(relaxng_doc)
            # If this doesn't raise an exception, the schema is valid
        except etree.RelaxNGParseError as e:
            raise ValueError(f"Invalid Relax NG schema: {e}")

    def find_root_element(self) -> str:
        """Find root element in Relax NG schema."""
        if not self.root_pattern:
            raise ValueError("No root pattern found in schema")

        # Look for first element in start pattern
        rng_ns = "http://relaxng.org/ns/structure/1.0"
        root = self.schema_doc.getroot()
        ns_map = root.nsmap

        rng_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == rng_ns:
                rng_prefix = prefix
                break

        if rng_prefix:
            ns = {rng_prefix: rng_ns}
            elem_xpath = f'.//{rng_prefix}:element'
            elements = self.root_pattern.xpath(elem_xpath, namespaces=ns)
        else:
            elements = self.root_pattern.xpath('.//element')

        if elements:
            return elements[0].get('name', 'root')

        return 'root'

    def generate_element(self, element_def, parent_element: ET.Element, depth: int = 0) -> None:
        """Generate XML element from Relax NG pattern."""
        if depth > 10:
            raise CircularReferenceError("Maximum recursion depth exceeded")

        # This is a simplified implementation
        # Real implementation would handle all Relax NG constructs

        rng_ns = "http://relaxng.org/ns/structure/1.0"
        root = self.schema_doc.getroot()
        ns_map = root.nsmap

        rng_prefix = None
        for prefix, namespace in ns_map.items():
            if namespace == rng_ns:
                rng_prefix = prefix
                break

        if rng_prefix:
            ns = {rng_prefix: rng_ns}
            prefix = f"{rng_prefix}:"
        else:
            ns = {}
            prefix = ""

        # Handle different Relax NG constructs
        if element_def is None:
            # Generate from root pattern
            element_def = self.root_pattern

        tag = element_def.tag
        if tag.endswith('}element') or tag == 'element':
            # Generate element
            name = element_def.get('name', 'element')
            elem = ET.SubElement(parent_element, name)

            # Generate content
            self._generate_content(element_def, elem, depth + 1)

        elif tag.endswith('}group') or tag == 'group':
            # Generate all children in group
            for child in element_def:
                self.generate_element(child, parent_element, depth + 1)

        elif tag.endswith('}choice') or tag == 'choice':
            # Generate one random choice
            children = list(element_def)
            if children:
                chosen = random.choice(children)
                self.generate_element(chosen, parent_element, depth + 1)

        elif tag.endswith('}optional') or tag == 'optional':
            # Maybe generate content
            if random.choice([True, False]):
                for child in element_def:
                    self.generate_element(child, parent_element, depth + 1)

        elif tag.endswith('}zeroOrMore') or tag == 'zeroOrMore':
            # Generate 0-3 repetitions
            count = random.randint(0, 3)
            for _ in range(count):
                for child in element_def:
                    self.generate_element(child, parent_element, depth + 1)

        elif tag.endswith('}oneOrMore') or tag == 'oneOrMore':
            # Generate 1-3 repetitions
            count = random.randint(1, 3)
            for _ in range(count):
                for child in element_def:
                    self.generate_element(child, parent_element, depth + 1)

        elif tag.endswith('}ref') or tag == 'ref':
            # Reference to named pattern
            name = element_def.get('name')
            if name in self.patterns:
                self.generate_element(self.patterns[name], parent_element, depth + 1)

        # Handle other Relax NG constructs...

    def _generate_content(self, element_def, elem: ET.Element, depth: int):
        """Generate content for an element."""
        # Look for text content or child elements
        for child in element_def:
            if child.tag.endswith('}text') or child.tag == 'text':
                elem.text = self.fake.text(max_nb_chars=50)
            elif child.tag.endswith('}data') or child.tag == 'data':
                # Generate data based on type
                data_type = child.get('type', 'string')
                elem.text = self._generate_data_for_type(data_type)
            else:
                self.generate_element(child, elem, depth)

    def _generate_data_for_type(self, data_type: str) -> str:
        """Generate fake data for Relax NG data types."""
        if data_type == 'string':
            return self.fake.text(max_nb_chars=50)
        elif data_type == 'int':
            return str(random.randint(1, 1000))
        elif data_type == 'date':
            return self.fake.date()
        # Add more data types as needed
        else:
            return self.fake.word()

    def check_circular_references(self) -> None:
        """Check for circular references in Relax NG schema."""
        # Implementation would be similar to XSD but adapted for Relax NG patterns
        visited = set()
        path = []

        for pattern_name, pattern_def in self.patterns.items():
            if self._has_circular_reference_rng(pattern_def, visited, path, pattern_name):
                cycle_path = " -> ".join(path)
                raise CircularReferenceError(f"Circular reference detected: {cycle_path}")

    def _has_circular_reference_rng(self, pattern_def, visited: Set[str], path: List[str], pattern_id: str) -> bool:
        """Check for circular references in Relax NG patterns."""
        if pattern_id in visited:
            path.append(pattern_id)
            return True

        visited.add(pattern_id)
        path.append(pattern_id)

        # Check for ref elements that might create cycles
        for child in pattern_def:
            if child.tag.endswith('}ref') or child.tag == 'ref':
                ref_name = child.get('name')
                if ref_name in self.patterns:
                    if self._has_circular_reference_rng(self.patterns[ref_name], visited, path, ref_name):
                        return True

        path.pop()
        visited.remove(pattern_id)
        return False


class RelaxNGCompactParser(SchemaParser):
    """Parser for Relax NG Compact Syntax."""

    def __init__(self):
        self.patterns = {}
        self.fake = Faker()

    def parse_schema(self, schema_path: str) -> None:
        """Parse Relax NG Compact syntax."""
        # This would require a compact syntax parser
        # For now, convert to XML syntax using trang or similar tool
        raise NotImplementedError("Relax NG Compact syntax not yet implemented")

    def validate_schema(self) -> None:
        raise NotImplementedError("Relax NG Compact syntax not yet implemented")

    def find_root_element(self) -> str:
        raise NotImplementedError("Relax NG Compact syntax not yet implemented")

    def generate_element(self, element_def, parent_element: ET.Element, depth: int = 0) -> None:
        raise NotImplementedError("Relax NG Compact syntax not yet implemented")

    def check_circular_references(self) -> None:
        raise NotImplementedError("Relax NG Compact syntax not yet implemented")


# Updated main function
def generate_xml_from_schema(schema_path: str, root_element: str = None, output_path: str = None,
                             schema_format: str = 'auto') -> str:
    """
    Generate XML file with fake data based on schema.

    Args:
        schema_path: Path to the schema file
        root_element: Name of the root element (optional)
        output_path: Path where to save the generated XML file (optional)
        schema_format: Schema format ('xsd', 'relaxng', 'relaxng_compact', or 'auto')

    Returns:
        Generated XML as string or path to saved file
    """
    generator = XMLGenerator(schema_path, schema_format)
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
            except Exception as e:
                print(e)
