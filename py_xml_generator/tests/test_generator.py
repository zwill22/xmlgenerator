import pytest
from pyxmlgenerator import generate as generate_xml

def test_empty():
    with pytest.raises(OSError) as e:
        generate_xml("")

    assert str(e.value) == "Oh no!"
