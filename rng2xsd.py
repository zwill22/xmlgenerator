import subprocess
import os
import tempfile
import shutil
from pathlib import Path


class RelaxNGToXSDConverter:
    def __init__(self, trang_jar_path=None):
        self.trang_jar_path = trang_jar_path or self._find_trang()

    def _find_trang(self):
        """Try to find Trang JAR in common locations"""
        possible_paths = [
            'trang.jar',
            '/usr/share/java/trang.jar',
            '/usr/local/share/java/trang.jar',
            os.path.expanduser('~/trang/trang.jar'),
        ]

        for path in possible_paths:
            if os.path.exists(path):
                return path

        return None

    def convert(self, relaxng_path, xsd_output_path=None):
        """
        Convert RelaxNG to XSD using the best available method
        """
        if self.trang_jar_path and shutil.which('java'):
            return self._convert_with_trang(relaxng_path, xsd_output_path)
        else:
            raise Exception(
                "Trang not found. Please install Trang and Java, or specify trang_jar_path. "
                "Download from: https://github.com/relaxng/jing-trang/releases"
            )

    def _convert_with_trang(self, relaxng_path, xsd_output_path):
        """Convert using Trang"""
        if xsd_output_path is None:
            xsd_output_path = Path(relaxng_path).with_suffix('.xsd')

        cmd = ['java', '-jar', self.trang_jar_path, relaxng_path, str(xsd_output_path)]

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            return str(xsd_output_path)
        except subprocess.CalledProcessError as e:
            raise Exception(f"Trang conversion failed: {e.stderr}")


# Usage example
def process_schema(schema_path):
    """Process a schema file, converting RelaxNG to XSD if necessary"""
    schema_path = Path(schema_path)

    if schema_path.suffix.lower() in ['.rng', '.rnc']:
        # Convert RelaxNG to XSD
        converter = RelaxNGToXSDConverter()
        xsd_path = converter.convert(str(schema_path))
        return xsd_path
    elif schema_path.suffix.lower() == '.xsd':
        # Already XSD
        return str(schema_path)
    else:
        raise ValueError(f"Unsupported schema format: {schema_path.suffix}")
