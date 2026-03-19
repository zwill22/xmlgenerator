import os
from io import BytesIO
from pathlib import Path

import pytest
import pyxmlgenerator
from lxml import etree
from xmlschema import XMLSchemaParseError, XMLSchemaModelError, XMLSchema10, XMLSchema11
from xmlschema.exceptions import XMLResourceParseError


def get_project_root() -> Path:
    """
    Get the project root (defined as location of `.git` directory)

    :return: Path
    """
    return next(
        p for p in Path(__file__).parents
        if (p / '.git').exists()
    )


def setup_xmlschema11(filepath):
    try:
        schema = XMLSchema11(filepath)
    except XMLSchemaParseError:
        schema = None
    except XMLSchemaModelError:
        schema = None

    return schema


class Validator:
    def __init__(self, filepath: str):
        try:
            schema = XMLSchema10(filepath)
        except XMLSchemaModelError:
            schema = setup_xmlschema11(filepath)
        except XMLSchemaParseError:
            schema = setup_xmlschema11(filepath)

        self.xmlschema = schema

        schema_doc = etree.parse(filepath)
        try:
            lxml_schema = etree.XMLSchema(schema_doc)
        except etree.XMLSchemaParseError:
            lxml_schema = None

        self.lxml_schema = lxml_schema

        if self.xmlschema is None and self.lxml_schema is None:
            pytest.xfail("Failed to setup schema")

    def print_output(self, xml: str):
        print("XSD")
        print(self.xmlschema.get_text())
        print("XML")
        print(xml)
        print()

    def validate(self, xml: str):
        xmlschema_valid = False
        errors = []
        if self.xmlschema:
            try:
                xmlschema_valid = self.xmlschema.is_valid(xml)
            except XMLResourceParseError as e:
                xmlschema_valid = False
                errors.append(e)

        lxml_schema_valid = False
        if self.lxml_schema:
            try:
                doc = etree.parse(BytesIO(xml.encode()))
                self.lxml_schema.assertValid(doc)
                lxml_schema_valid = True
            except etree.XMLSyntaxError as e:
                lxml_schema_valid = False
                errors.append(e)
            except etree.DocumentInvalid:
                lxml_schema_valid = False

        if not xmlschema_valid:
            if not lxml_schema_valid:
                return False, errors

            pytest.xfail("XMLSchema failed to validate output")

        if not lxml_schema_valid:
            pytest.xfail("LXML failed to validate output")

        return True, errors


def run_generator(xml_generator, filepath) -> str:
    try:
        result: str = xml_generator.generate(filepath)
    except pyxmlgenerator.NoElementsError:
        pytest.xfail("XSD does not contain any elements")
    except pyxmlgenerator.ImplementationError as e:
        pytest.skip(f"Unimplemented feature {e}")
    except pyxmlgenerator.MultipleXSDRootsError:
        pytest.xfail("XSD contains multiple roots")
    except pyxmlgenerator.XSDParserError:
        pytest.xfail("Unable to parse XSD")
    except pyxmlgenerator.InfiniteRecursionError:
        pytest.xfail("Infinite recursion found in XSD")
    except pyxmlgenerator.InvalidXSDNameError:
        pytest.xfail("Invalid XSD naming")
    except pyxmlgenerator.DataTypesFormatError:
        pytest.xfail("Data types format error")
    except pyxmlgenerator.LineEndingsError:
        pytest.xfail("Line endings error")
    except pyxmlgenerator.InvalidXSDVersionError:
        pytest.xfail("Invalid XSD version")
    except pyxmlgenerator.NoIndependentElementsError:
        pytest.xfail("No independent elements in schema")
    except pyxmlgenerator.IncompatiblePatternError:
        pytest.xfail("XSD defines a pattern that is incompatible with the base type")
    except pyxmlgenerator.XSDEncodingError:
        pytest.xfail("No encoding specified in XSD")

    return result


def validate_output(xml_generator, input_file: Path | str):
    """
    Validate whether py-xmlgenerator generates output that matches the input schema

    :param xml_generator: XMLGenerator fixture
    :param input_file: Input schema passed to py-xmlgenerator
    
    :raise x
    :return: 
    """

    filepath = str(input_file)
    file_dir = Path(input_file).parent

    cwd = Path.cwd()
    os.chdir(file_dir)

    validator = Validator(filepath)

    result = run_generator(xml_generator, filepath)

    valid, errors = validator.validate(result)

    os.chdir(cwd)

    try:
        assert valid
    except AssertionError as e:
        print("Invalid XML")
        validator.print_output(result)
        print("Errors:")

        if len(errors) > 1:
            for i in range(1, len(errors)):
                print(errors[i])

        if len(errors) == 1:
            raise errors[0]

        raise e
