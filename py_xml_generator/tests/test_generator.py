import pytest

from pyxmlgenerator import InvalidPathError

def test_empty(xml_generator):

    with pytest.raises(InvalidPathError) as e:
        xml_generator.generate("")

    assert str(e.value) == "XMLGenerator panic error: Filepath has no parent"
