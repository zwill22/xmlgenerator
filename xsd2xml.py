import random
import string
from datetime import datetime, timedelta
from decimal import Decimal
from lxml import etree
from faker import Faker
import xml.etree.ElementTree as ET

fake = Faker()


def generate_xml_from_schema(schema_path, output_path=None, root_element=None, max_occurs_limit=3):
    """
    Generate an XML file with fake data based on an XSD schema.

    Args:
        schema_path: Path to the XSD schema file
        output_path: Path where the generated XML will be saved (optional)
        root_element: Name of the root element to generate (if schema has multiple)
        max_occurs_limit: Maximum number of elements to generate for unbounded elements

    Returns:
        Generated XML as string
    """

    # Parse the schema
    with open(schema_path, 'r') as f:
        schema_doc = etree.parse(f)

    schema = etree.XMLSchema(schema_doc)

    # Get namespace
    schema_root = schema_doc.getroot()
    target_namespace = schema_root.get('targetNamespace', '')

    # Find root element
    if root_element is None:
        # Try to find the first global element
        elements = schema_doc.xpath('//xs:element[@name]',
                                    namespaces={'xs': 'http://www.w3.org/2001/XMLSchema'})
        if elements:
            root_element = elements[0].get('name')
        else:
            raise ValueError("No root element found in schema")

    # Generate the XML
    nsmap = {None: target_namespace} if target_namespace else None
    root = etree.Element(root_element, nsmap=nsmap)

    # Find the root element definition in schema
    root_def = schema_doc.xpath(f'//xs:element[@name="{root_element}"]',
                                namespaces={'xs': 'http://www.w3.org/2001/XMLSchema'})[0]

    # Process the root element
    process_element(root, root_def, schema_doc, target_namespace, max_occurs_limit)

    # Create the XML tree
    tree = etree.ElementTree(root)

    # Generate XML string
    xml_string = etree.tostring(tree, pretty_print=True,
                                xml_declaration=True, encoding='UTF-8').decode('utf-8')

    # Save to file if output path provided
    if output_path:
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(xml_string)

    return xml_string


def process_element(parent, element_def, schema_doc, namespace, max_occurs_limit):
    """Process an element definition and add appropriate content."""

    # Check if element has a type attribute
    type_attr = element_def.get('type')

    if type_attr:
        # Element has a type reference
        if ':' in type_attr:
            # Remove namespace prefix if present
            type_name = type_attr.split(':')[1]
        else:
            type_name = type_attr

        # Check if it's a built-in type
        if is_builtin_type(type_name):
            # Generate content for simple type
            content = generate_fake_data(type_name)
            if content is not None:
                parent.text = str(content)
        else:
            # Look for complex type definition
            complex_type = schema_doc.xpath(f'//xs:complexType[@name="{type_name}"]',
                                            namespaces={'xs': 'http://www.w3.org/2001/XMLSchema'})
            if complex_type:
                process_complex_type(parent, complex_type[0], schema_doc, namespace, max_occurs_limit)
    else:
        # Element has inline type definition
        complex_type = element_def.find('.//{http://www.w3.org/2001/XMLSchema}complexType')
        simple_type = element_def.find('.//{http://www.w3.org/2001/XMLSchema}simpleType')

        if complex_type is not None:
            process_complex_type(parent, complex_type, schema_doc, namespace, max_occurs_limit)
        elif simple_type is not None:
            # Process simple type
            restriction = simple_type.find('.//{http://www.w3.org/2001/XMLSchema}restriction')
            if restriction is not None:
                base_type = restriction.get('base', 'string').split(':')[-1]
                content = generate_fake_data(base_type)
                if content is not None:
                    parent.text = str(content)


def process_complex_type(parent, complex_type, schema_doc, namespace, max_occurs_limit):
    """Process a complex type definition."""

    # Process sequences
    sequences = complex_type.findall('.//{http://www.w3.org/2001/XMLSchema}sequence')
    for sequence in sequences:
        process_sequence(parent, sequence, schema_doc, namespace, max_occurs_limit)

    # Process choices
    choices = complex_type.findall('.//{http://www.w3.org/2001/XMLSchema}choice')
    for choice in choices:
        process_choice(parent, choice, schema_doc, namespace, max_occurs_limit)

    # Process attributes
    attributes = complex_type.findall('.//{http://www.w3.org/2001/XMLSchema}attribute')
    for attr in attributes:
        process_attribute(parent, attr, schema_doc)


def process_sequence(parent, sequence, schema_doc, namespace, max_occurs_limit):
    """Process a sequence of elements."""

    elements = sequence.findall('.//{http://www.w3.org/2001/XMLSchema}element')

    for elem in elements:
        name = elem.get('name')
        ref = elem.get('ref')

        if ref:
            # Element reference
            ref_name = ref.split(':')[-1]
            # Find the referenced element
            ref_elem = schema_doc.xpath(f'//xs:element[@name="{ref_name}"]',
                                        namespaces={'xs': 'http://www.w3.org/2001/XMLSchema'})
            if ref_elem:
                elem = ref_elem[0]
                name = ref_name

        if name:
            # Determine how many times to generate this element
            min_occurs = int(elem.get('minOccurs', '1'))
            max_occurs = elem.get('maxOccurs', '1')

            if max_occurs == 'unbounded':
                max_occurs = max_occurs_limit
            else:
                max_occurs = int(max_occurs)

            # Generate between minOccurs and maxOccurs instances
            count = random.randint(min_occurs, max_occurs)

            for _ in range(count):
                # Create the element
                if namespace:
                    child = etree.SubElement(parent, f"{{{namespace}}}{name}")
                else:
                    child = etree.SubElement(parent, name)

                # Process the element content
                process_element(child, elem, schema_doc, namespace, max_occurs_limit)


def process_choice(parent, choice, schema_doc, namespace, max_occurs_limit):
    """Process a choice - select one of the options."""

    elements = choice.findall('.//{http://www.w3.org/2001/XMLSchema}element')

    if elements:
        # Choose one element randomly
        chosen = random.choice(elements)

        name = chosen.get('name')
        ref = chosen.get('ref')

        if ref:
            ref_name = ref.split(':')[-1]
            ref_elem = schema_doc.xpath(f'//xs:element[@name="{ref_name}"]',
                                        namespaces={'xs': 'http://www.w3.org/2001/XMLSchema'})
            if ref_elem:
                chosen = ref_elem[0]
                name = ref_name

        if name:
            if namespace:
                child = etree.SubElement(parent, f"{{{namespace}}}{name}")
            else:
                child = etree.SubElement(parent, name)

            process_element(child, chosen, schema_doc, namespace, max_occurs_limit)


def process_attribute(parent, attr_def, schema_doc):
    """Process an attribute definition."""

    name = attr_def.get('name')
    type_attr = attr_def.get('type', 'string')
    use = attr_def.get('use', 'optional')

    # Only add required attributes or randomly add optional ones
    if use == 'required' or (use == 'optional' and random.choice([True, False])):
        type_name = type_attr.split(':')[-1]
        value = generate_fake_data(type_name)
        if value is not None:
            parent.set(name, str(value))


def is_builtin_type(type_name):
    """Check if a type is a built-in XSD type."""
    builtin_types = {
        'string', 'boolean', 'decimal', 'float', 'double',
        'duration', 'dateTime', 'time', 'date', 'gYearMonth',
        'gYear', 'gMonthDay', 'gDay', 'gMonth', 'hexBinary',
        'base64Binary', 'anyURI', 'QName', 'NOTATION',
        'normalizedString', 'token', 'language', 'NMTOKEN',
        'NMTOKENS', 'Name', 'NCName', 'ID', 'IDREF', 'IDREFS',
        'ENTITY', 'ENTITIES', 'integer', 'nonPositiveInteger',
        'negativeInteger', 'long', 'int', 'short', 'byte',
        'nonNegativeInteger', 'unsignedLong', 'unsignedInt',
        'unsignedShort', 'unsignedByte', 'positiveInteger'
    }
    return type_name in builtin_types


def generate_fake_data(xsd_type):
    """Generate fake data based on XSD type."""

    if xsd_type == 'string':
        return fake.sentence(nb_words=3)
    elif xsd_type == 'normalizedString' or xsd_type == 'token':
        return fake.word()
    elif xsd_type == 'boolean':
        return random.choice(['true', 'false'])
    elif xsd_type in ['decimal', 'float', 'double']:
        return round(random.uniform(0, 1000), 2)
    elif xsd_type in ['integer', 'int']:
        return random.randint(-1000, 1000)
    elif xsd_type in ['positiveInteger', 'nonNegativeInteger', 'unsignedInt']:
        return random.randint(0, 1000)
    elif xsd_type == 'date':
        return fake.date()
    elif xsd_type == 'dateTime':
        return fake.date_time().isoformat()
    elif xsd_type == 'time':
        return fake.time()
    elif xsd_type == 'anyURI':
        return fake.url()
    elif xsd_type == 'NCName' or xsd_type == 'Name':
        return fake.word().replace(' ', '_')
    elif xsd_type == 'ID':
        return f"id_{fake.uuid4().split('-')[0]}"
    elif xsd_type == 'language':
        return random.choice(['en', 'es', 'fr', 'de', 'it'])
    elif xsd_type in ['long', 'short', 'byte']:
        return random.randint(-100, 100)
    elif xsd_type in ['unsignedLong', 'unsignedShort', 'unsignedByte']:
        return random.randint(0, 100)
    else:
        # Default to string for unknown types
        return fake.word()


# Example usage:
if __name__ == "__main__":
    # Example 1: Generate XML from schema and save to file
    xml_content = generate_xml_from_schema(
        schema_path="example.xsd",
        output_path="generated_test_data.xml",
        max_occurs_limit=2
    )
    print("Generated XML:")
    print(xml_content)
