import pytest
from pyxmlgenerator import generate as generate_xml

def test_empty():
    with pytest.raises(RuntimeError) as e:
        generate_xml("")

    assert str(e.value) == "XSD Parser encountered an error.\nXML Error: Unexpected event: Eof!; position=0"
