import subprocess
import tempfile
import os
from pathlib import Path


class SchemaConverter:
    def __init__(self, trang_path=None):
        self.trang_path = trang_path or self._find_trang()

    def _find_trang(self):
        """Try to find trang executable or JAR"""
        # Check if trang is in PATH
        try:
            subprocess.run(['trang', '--help'], capture_output=True, check=True)
            return 'trang'
        except:
            pass

        # Check common locations for trang.jar
        possible_paths = [
            'trang.jar',
            '/usr/local/lib/trang.jar',
            os.path.expanduser('~/lib/trang.jar'),
        ]

        for path in possible_paths:
            if os.path.exists(path):
                return path

        raise Exception("Trang not found. Please install trang or specify path to trang.jar")

    def convert_rng_to_xsd(self, rng_path, xsd_path=None):
        """Convert RelaxNG to XSD"""
        if xsd_path is None:
            # Generate output path by changing extension
            xsd_path = Path(rng_path).with_suffix('.xsd')

        if self.trang_path == 'trang':
            # Use installed trang command
            cmd = ['trang', rng_path, str(xsd_path)]
        else:
            # Use JAR file
            cmd = ['java', '-jar', self.trang_path, rng_path, str(xsd_path)]

        result = subprocess.run(cmd, capture_output=True, text=True)

        if result.returncode != 0:
            raise Exception(f"Conversion failed: {result.stderr}")

        return str(xsd_path)

    def convert_rng_to_xsd_temp(self, rng_content):
        """Convert RelaxNG content to XSD, returning the XSD content"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.rng', delete=False) as rng_file:
            rng_file.write(rng_content)
            rng_file.flush()

            with tempfile.NamedTemporaryFile(mode='r', suffix='.xsd', delete=False) as xsd_file:
                xsd_path = xsd_file.name

            try:
                self.convert_rng_to_xsd(rng_file.name, xsd_path)
                with open(xsd_path, 'r') as f:
                    xsd_content = f.read()
                return xsd_content
            finally:
                os.unlink(rng_file.name)
                if os.path.exists(xsd_path):
                    os.unlink(xsd_path)


if __name__ == '__main__':
    # Usage example
    converter = SchemaConverter()

    # Convert file to file
    converter.convert_rng_to_xsd('input.rng', 'output.xsd')

    # Convert with automatic output naming
    xsd_path = converter.convert_rng_to_xsd('schema.rng')  # Creates schema.xsd

    # Convert string content
    rng_content = """<?xml version="1.0" encoding="UTF-8"?>
    <element name="root" xmlns="http://relaxng.org/ns/structure/1.0">
      <text/>
    </element>"""
    xsd_content = converter.convert_rng_to_xsd_temp(rng_content)