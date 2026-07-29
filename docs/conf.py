import os
import sys

import pyxmlgenerator
import pyxsdtestdata

sys.path.insert(0, os.path.abspath(".."))

project = "pyxmlgenerator"
copyright = "2026, Z M Williams"
author = "Z M Williams"
release = pyxmlgenerator.version()

assert pyxsdtestdata.version() == pyxmlgenerator.version()

extensions = [
    "sphinx.ext.duration",
    "sphinx.ext.intersphinx",
    "sphinx.ext.napoleon",
    "sphinx.ext.autosectionlabel",
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "myst_parser",
]

autodoc_typehints = "description"

autodoc_member_order = "bysource"

napoleon_google_docstring = True

intersphinx_mapping = {
    "python": ("https://docs.python.org/3/", None),
    "sphinx": ("https://www.sphinx-doc.org/en/master/", None),
}
intersphinx_disabled_domains = ["std"]

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

html_theme = "sphinx_rtd_theme"
html_static_path = ["_static"]
html_context = {
    "display_github": True,
    "github_user": "zwill22",
    "github_repo": "xmlgenerator",
    "conf_py_path": "/docs/",
    "github_version": "main",
}

autosectionlabel_prefix_document = True
