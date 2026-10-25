#!/usr/bin/env python3
"""Validación estructural de los DOCX/ODT generados sin usar ApexDesk."""
from pathlib import Path
from xml.etree import ElementTree as ET
from zipfile import ZipFile

fixtures = Path(__file__).resolve().parents[1] / "fixtures"
w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
ot = "urn:oasis:names:tc:opendocument:xmlns:text:1.0"
os = "urn:oasis:names:tc:opendocument:xmlns:style:1.0"
fo = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
dc = "http://purl.org/dc/elements/1.1/"

def text(node):
    return "".join(node.itertext())

with ZipFile(fixtures / "externo.docx") as z:
    assert z.testzip() is None
    doc = ET.fromstring(z.read("word/document.xml"))
    paras = doc.findall(f".//{{{w}}}body/{{{w}}}p")
    assert [text(x) for x in paras] == [
        "Cabecera sin modificar", "Texto de ejemplo para comprobar el formato."
    ]
    assert paras[1].find(f"{{{w}}}r/{{{w}}}rPr/{{{w}}}b") is not None
    core = ET.fromstring(z.read("docProps/core.xml"))
    assert "Documento externo de prueba" in text(core)
print("OK: DOCX externo válido")

with ZipFile(fixtures / "externo.odt") as z:
    assert z.testzip() is None
    assert z.read("mimetype") == b"application/vnd.oasis.opendocument.text"
    root = ET.fromstring(z.read("content.xml"))
    paras = root.findall(f".//{{{ot}}}p")
    assert [text(x) for x in paras] == [
        "Cabecera sin modificar", "Texto de ejemplo para comprobar el formato."
    ]
    span = paras[1].find(f"{{{ot}}}span")
    assert span is not None and text(span) == "Texto"
    name = span.get(f"{{{ot}}}style-name")
    styles = ET.fromstring(z.read("styles.xml"))
    style = styles.find(f'.//{{{os}}}style[@{{{os}}}name="{name}"]')
    assert style is not None
    assert style.find(f"{{{os}}}text-properties").get(f"{{{fo}}}font-weight") == "bold"
    meta = ET.fromstring(z.read("meta.xml"))
    assert meta.find(f".//{{{dc}}}title").text == "Documento externo de prueba"
print("OK: ODT externo válido")
