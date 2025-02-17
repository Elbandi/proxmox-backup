#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# Proxmox Backup documentation build configuration file, originally
# created by sphinx-quickstart on Tue Feb 26 16:54:35 2019.
#
# This file is execfile()d with the current directory set to its
# containing dir.
#
# Note that not all possible configuration values are present in this
# autogenerated file.
#
# All configuration values have a default; values that are commented out
# serve to show the default.

# If extensions (or modules to document with autodoc) are in another directory,
# add these directories to sys.path here. If the directory is relative to the
# documentation root, use os.path.abspath to make it absolute, like shown here.
#
import os
import sys
# sys.path.insert(0, os.path.abspath('.'))

# custom extensions
sys.path.append(os.path.abspath("./_ext"))

# -- Implement custom formatter for code-blocks ---------------------------
#
# * use smaller font
# * avoid space between lines to nicely format utf8 tables

from sphinx.highlighting import PygmentsBridge
from pygments.formatters.latex import LatexFormatter

class CustomLatexFormatter(LatexFormatter):
    def __init__(self, **options):
        super(CustomLatexFormatter, self).__init__(**options)
        self.verboptions = r"formatcom=\footnotesize\relax\let\strut\empty"

PygmentsBridge.latex_formatter = CustomLatexFormatter

# -- General configuration ------------------------------------------------

# If your documentation needs a minimal Sphinx version, state it here.
#
# needs_sphinx = '1.0'

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.

extensions = ["sphinx.ext.graphviz", 'sphinx.ext.mathjax', "sphinx.ext.todo", "proxmox-scanrefs"]

todo_link_only = True

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# The suffix(es) of source filenames.
# You can specify multiple suffix as a list of string:
#
# source_suffix = ['.rst', '.md']
source_suffix = '.rst'

# The encoding of source files.
#
# source_encoding = 'utf-8-sig'

# The master toctree document.
master_doc = 'index'

# General information about the project.
project = 'Proxmox Backup'
copyright = '2019-2022, Proxmox Server Solutions GmbH'
author = 'Proxmox Support Team'

# The version info for the project you're documenting acts as a replacement for
# |version| and |release|, also used in various other places throughout the
# built documents.
#
# The short X.Y version.
vstr = lambda s: '<devbuild>' if s is None else str(s)

version = vstr(os.getenv('DEB_VERSION_UPSTREAM'))
# The full version, including alpha/beta/rc tags.
release = vstr(os.getenv('DEB_VERSION'))

epilog_file = open('epilog.rst', 'r')
rst_epilog = epilog_file.read()
rst_epilog += f"\n..  |VERSION| replace:: {version}"
rst_epilog += f"\n..  |pbs-copyright| replace:: Copyright (C) {copyright}"

man_pages = [
    # CLI
    ('proxmox-backup-client/man1', 'proxmox-backup-client', 'Command line tool for Backup and Restore', [author], 1),
    ('proxmox-backup-manager/man1', 'proxmox-backup-manager', 'Command line tool to manage and configure the backup server.', [author], 1),
    ('proxmox-backup-debug/man1', 'proxmox-backup-debug', 'Debugging command line tool for Backup and Restore', [author], 1),
    ('proxmox-backup-proxy/man1', 'proxmox-backup-proxy', 'Proxmox Backup Public API Server', [author], 1),
    ('proxmox-backup/man1', 'proxmox-backup', 'Proxmox Backup Local API Server', [author], 1),
    ('proxmox-file-restore/man1', 'proxmox-file-restore', 'CLI tool for restoring files and directories from Proxmox Backup Server archives', [author], 1),
    ('proxmox-tape/man1', 'proxmox-tape', 'Proxmox Tape Backup CLI Tool', [author], 1),
    ('pxar/man1', 'pxar', 'Proxmox File Archive CLI Tool', [author], 1),
    ('pmt/man1', 'pmt', 'Control Linux Tape Devices', [author], 1),
    ('pmtx/man1', 'pmtx', 'Control SCSI media changer devices (tape autoloaders)', [author], 1),
    ('pbs2to3/man1', 'pbs2to3', 'Proxmox Backup Server upgrade checker script for 2.4+ to current 3.x major upgrades', [author], 1),
    # configs
    ('config/acl/man5', 'acl.cfg', 'Access Control Configuration', [author], 5),
    ('config/datastore/man5', 'datastore.cfg', 'Datastore Configuration', [author], 5),
    ('config/domains/man5', 'domains.cfg', 'Realm Configuration', [author], 5),
    ('config/media-pool/man5', 'media-pool.cfg', 'Media Pool Configuration', [author], 5),
    ('config/remote/man5', 'remote.cfg', 'Remote Server Configuration', [author], 5),
    ('config/sync/man5', 'sync.cfg', 'Synchronization Job Configuration', [author], 5),
    ('config/tape-job/man5', 'tape-job.cfg', 'Tape Job Configuration', [author], 5),
    ('config/tape/man5', 'tape.cfg', 'Tape Drive and Changer Configuration', [author], 5),
    ('config/user/man5', 'user.cfg', 'User Configuration', [author], 5),
    ('config/verification/man5', 'verification.cfg', 'Verification Job Configuration', [author], 5),
]


# The language for content autogenerated by Sphinx. Refer to documentation
# for a list of supported languages.
#
# This is also used if you do content translation via gettext catalogs.
# Usually you set "language" from the command line for these cases.
language = None

# There are two options for replacing |today|: either, you set today to some
# non-false value, then it is used:
# today = ''
#
# Else, today_fmt is used as the format for a strftime call.
today_fmt = '%A, %d %B %Y'

suppress_warnings = [ 'toc.excluded' ]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This patterns also effect to html_static_path and html_extra_path
exclude_patterns = [
    '_build', 'Thumbs.db', '.DS_Store',
    'certificate-management.rst',
    'epilog.rst',
    'pbs-copyright.rst',
    'local-zfs.rst',
    'package-repositories.rst',
    'system-booting.rst',
    'traffic-control.rst',
]

# The reST default role (used for this markup: `text`) to use for all
# documents.
#
# default_role = None

# If true, '()' will be appended to :func: etc. cross-reference text.
#
# add_function_parentheses = True

# If true, the current module name will be prepended to all description
# unit titles (such as .. function::).
#
# add_module_names = True

# If true, sectionauthor and moduleauthor directives will be shown in the
# output. They are ignored by default.
#
# show_authors = False

# The name of the Pygments (syntax highlighting) style to use.
pygments_style = 'sphinx'

# A list of ignored prefixes for module index sorting.
# modindex_common_prefix = []

# If true, keep warnings as "system message" paragraphs in the built documents.
# keep_warnings = False

# If true, `todo` and `todoList` produce output, else they produce nothing.
todo_include_todos = not tags.has('release')


# -- Options for HTML output ----------------------------------------------

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
#
html_theme = 'alabaster'

# Theme options are theme-specific and customize the look and feel of a theme
# further.  For a list of options available for each theme, see the
# documentation.
#
html_theme_options = {
    'fixed_sidebar': True,
    'sidebar_includehidden': False,
    'sidebar_collapse': False,
    'globaltoc_collapse': False,
    'show_relbar_bottom': True,
    'show_powered_by': False,

    'extra_nav_links': {
        'Proxmox Homepage': 'https://proxmox.com',
        'PDF': 'proxmox-backup.pdf',
        'API Viewer' : 'api-viewer/index.html',
        'Prune Simulator' : 'prune-simulator/index.html',
        'LTO Barcode Generator' : 'lto-barcode/index.html',
    },

    'sidebar_width': '320px',
    'page_width': '1320px',
    # font styles
    'head_font_family': 'Lato, sans-serif',
    'caption_font_family': 'Lato, sans-serif',
    'caption_font_size': '20px',
    'font_family': 'Open Sans, sans-serif',
}

# Alabaster theme recommends setting this fixed.
# If you switch theme this needs to removed, probably.
html_sidebars = {
    '**': [
        'sidebar-header.html',
        'searchbox.html',
        'navigation.html',
        'relations.html',
    ],

    'index': [
        'sidebar-header.html',
        'searchbox.html',
        'index-sidebar.html',
    ]
}


# Add any paths that contain custom themes here, relative to this directory.
# html_theme_path = []

# The name for this set of Sphinx documents.
# "<project> v<release> documentation" by default.
#
# html_title = 'Proxmox Backup v1.0-1'

# A shorter title for the navigation bar.  Default is the same as html_title.
#
# html_short_title = None

# The name of an image file (relative to this directory) to place at the top
# of the sidebar.
#
#html_logo = 'images/proxmox-logo.svg' # replaced by html_theme_options.logo

# The name of an image file (relative to this directory) to use as a favicon of
# the docs.  This file should be a Windows icon file (.ico) being 16x16 or 32x32
# pixels large.
#
html_favicon = 'images/favicon.ico'

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']

html_js_files = [
    'custom.js',
]

# Add any extra paths that contain custom files (such as robots.txt or
# .htaccess) here, relative to this directory. These files are copied
# directly to the root of the documentation.
#
# html_extra_path = []

# If not None, a 'Last updated on:' timestamp is inserted at every page
# bottom, using the given strftime format.
# The empty string is equivalent to '%b %d, %Y'.
#
# html_last_updated_fmt = None

# We need to disable smatquotes, else Option Lists do not display long options
smartquotes = False

# Additional templates that should be rendered to pages, maps page names to
# template names.
#
# html_additional_pages = {}

# If false, no module index is generated.
#
# html_domain_indices = True

# If false, no index is generated.
#
# html_use_index = True

# If true, the index is split into individual pages for each letter.
#
# html_split_index = False

# If true, links to the reST sources are added to the pages.
#
html_show_sourcelink = False

# If true, "Created using Sphinx" is shown in the HTML footer. Default is True.
#
# html_show_sphinx = True

# If true, "(C) Copyright ..." is shown in the HTML footer. Default is True.
#
# html_show_copyright = True

# If true, an OpenSearch description file will be output, and all pages will
# contain a <link> tag referring to it.  The value of this option must be the
# base URL from which the finished HTML is served.
#
# html_use_opensearch = ''

# This is the file name suffix for HTML files (e.g. ".xhtml").
# html_file_suffix = None

# Language to be used for generating the HTML full-text search index.
# Sphinx supports the following languages:
#   'da', 'de', 'en', 'es', 'fi', 'fr', 'h', 'it', 'ja'
#   'nl', 'no', 'pt', 'ro', 'r', 'sv', 'tr', 'zh'
#
# html_search_language = 'en'

# A dictionary with options for the search language support, empty by default.
# 'ja' uses this config value.
# 'zh' user can custom change `jieba` dictionary path.
#
# html_search_options = {'type': 'default'}

# The name of a javascript file (relative to the configuration directory) that
# implements a search results scorer. If empty, the default will be used.
#
# html_search_scorer = 'scorer.js'

# Output file base name for HTML help builder.
htmlhelp_basename = 'ProxmoxBackupdoc'

# use local mathjax package, symlink comes from debian/proxmox-backup-docs.links
mathjax_path = "mathjax/MathJax.js?config=TeX-AMS-MML_HTMLorMML"

# -- Options for LaTeX output ---------------------------------------------

latex_engine = 'xelatex'

latex_elements = {
    'fontenc': '\\usepackage{fontspec}',

     # The paper size ('letterpaper' or 'a4paper').
     #
     'papersize': 'a4paper',

     # The font size ('10pt', '11pt' or '12pt').
     #
     'pointsize': '10pt',

    'fontpkg': r'''
\setmainfont{Open Sans}
\setsansfont{Lato}
\setmonofont{DejaVu Sans Mono}
''',

     # Additional stuff for the LaTeX preamble.
     #
     # 'preamble': '',

     # Latex figure (float) alignment
     #
     # 'figure_align': 'htbp',
}

# Grouping the document tree into LaTeX files. List of tuples
# (source start file, target name, title,
#  author, documentclass [howto, manual, or own class]).
latex_documents = [
    (master_doc, 'ProxmoxBackup.tex', 'Proxmox Backup Documentation',
     'Proxmox Support Team', 'manual'),
]

# The name of an image file (relative to this directory) to place at the top of
# the title page.
#
#
# Note: newer sphinx 1.6 should be able to convert .svg to .png
# automatically using sphinx.ext.imgconverter
latex_logo = "images/proxmox-logo.png"

# For "manual" documents, if this is true, then toplevel headings are parts,
# not chapters.
#
# latex_use_parts = False

# If true, show page references after internal links.
#
# latex_show_pagerefs = False

# If true, show URL addresses after external links.
#
# latex_show_urls = False

# Documents to append as an appendix to all manuals.
#
# latex_appendices = []

# It false, will not define \strong, \code, 	itleref, \crossref ... but only
# \sphinxstrong, ..., \sphinxtitleref, ... To help avoid clash with user added
# packages.
#
# latex_keep_old_macro_names = True

# If false, no module index is generated.
#
# latex_domain_indices = True


# -- Options for Epub output ----------------------------------------------

# Bibliographic Dublin Core info.
epub_title = project
epub_author = author
epub_publisher = author
epub_copyright = copyright

# The basename for the epub file. It defaults to the project name.
# epub_basename = project

# The HTML theme for the epub output. Since the default themes are not
# optimized for small screen space, using the same theme for HTML and epub
# output is usually not wise. This defaults to 'epub', a theme designed to save
# visual space.
#
# epub_theme = 'epub'

# The language of the text. It defaults to the language option
# or 'en' if the language is not set.
#
# epub_language = ''

# The scheme of the identifier. Typical schemes are ISBN or URL.
# epub_scheme = ''

# The unique identifier of the text. This can be a ISBN number
# or the project homepage.
#
# epub_identifier = ''

# A unique identification for the text.
#
# epub_uid = ''

# A tuple containing the cover image and cover page html template filenames.
#
# epub_cover = ()

# A sequence of (type, uri, title) tuples for the guide element of content.opf.
#
# epub_guide = ()

# HTML files that should be inserted before the pages created by sphinx.
# The format is a list of tuples containing the path and title.
#
# epub_pre_files = []

# HTML files that should be inserted after the pages created by sphinx.
# The format is a list of tuples containing the path and title.
#
# epub_post_files = []

# A list of files that should not be packed into the epub file.
epub_exclude_files = ['search.html']

# The depth of the table of contents in toc.ncx.
#
# epub_tocdepth = 3

# Allow duplicate toc entries.
#
# epub_tocdup = True

# Choose between 'default' and 'includehidden'.
#
# epub_tocscope = 'default'

# Fix unsupported image types using the Pillow.
#
# epub_fix_images = False

# Scale large images.
#
# epub_max_image_width = 0

# How to display URL addresses: 'footnote', 'no', or 'inline'.
#
# epub_show_urls = 'inline'

# If false, no index is generated.
#
# epub_use_index = True
