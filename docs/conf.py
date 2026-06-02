"""Sphinx configuration for conda-express documentation."""

project = html_title = "conda-express"
copyright = "2026, Jannis Leidel"
author = "Jannis Leidel"

extensions = [
    "myst_parser",
    "sphinx.ext.intersphinx",
    "sphinx_copybutton",
    "sphinx_design",
    "sphinx_reredirects",
    "sphinx_sitemap",
]

myst_enable_extensions = [
    "colon_fence",
    "deflist",
    "fieldlist",
    "tasklist",
]

html_theme = "conda_sphinx_theme"

html_theme_options = {
    "icon_links": [
        {
            "name": "GitHub",
            "url": "https://github.com/jezdez/conda-express",
            "icon": "fa-brands fa-square-github",
            "type": "fontawesome",
        },
        {
            "name": "PyPI",
            "url": "https://pypi.org/project/conda-express/",
            "icon": "fa-brands fa-python",
            "type": "fontawesome",
        },
    ],
}

html_context = {
    "github_user": "jezdez",
    "github_repo": "conda-express",
    "github_version": "main",
    "doc_path": "docs",
}

html_static_path = ["_static"]
html_extra_path = ["../scripts/get-cx.sh", "../scripts/get-cx.ps1"]
html_css_files = ["css/custom.css"]

html_baseurl = "https://jezdez.github.io/conda-express/"

intersphinx_mapping = {
    "conda-ship": ("https://jezdez.github.io/conda-ship/", None),
}

exclude_patterns = ["_build"]
