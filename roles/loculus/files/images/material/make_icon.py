"""Turn a ViralZone virion SVG into a label-free 3:2 organism image.

usage: make_icon.py SRC DST --drop ID [ID...] [--highlight ID [ID...]]

- removes the label groups given by --drop (outlined glyphs + arrows)
- with --highlight, fades every other top-level element to light grey
- sets a 3:2 viewBox (the site's 240x160 organism images) centred on the
  drawing, so nothing is clipped
"""
import argparse, os, re, struct, subprocess, tempfile, zlib
import xml.etree.ElementTree as ET

NS = "http://www.w3.org/2000/svg"
ET.register_namespace("", NS)
ET.register_namespace("xlink", "http://www.w3.org/1999/xlink")
MARGIN = 1.06  # canvas padding around the drawing's bounding box
ASPECT = 240 / 160  # organism cards on db.wasap.genspectrum.org

ap = argparse.ArgumentParser()
ap.add_argument("src"); ap.add_argument("dst")
ap.add_argument("--drop", nargs="+", required=True)
ap.add_argument("--highlight", nargs="*", default=[])
a = ap.parse_args()

text = open(a.src).read()
text = re.sub(r"\n<(\s*<)", r"\n\1", text)  # stray '<' in some upstream files
root = ET.fromstring(text)

for el in list(root):
    if el.get("id") in a.drop:
        root.remove(el)
for el in list(root):  # empty leftover groups
    if el.tag == f"{{{NS}}}g" and len(el) == 0:
        root.remove(el)
missing = set(a.drop + a.highlight) - {e.get("id") for e in root.iter()} - set(a.drop)
assert not missing, f"unknown ids: {missing}"

if a.highlight:
    defs = ET.Element(f"{{{NS}}}defs")
    f = ET.SubElement(defs, f"{{{NS}}}filter", id="faded", x="0", y="0", width="1", height="1")
    ET.SubElement(f, f"{{{NS}}}feColorMatrix", type="saturate", values="0")
    ct = ET.SubElement(f, f"{{{NS}}}feComponentTransfer")
    for ch in "RGB":
        ET.SubElement(ct, f"{{{NS}}}feFunc{ch}", type="linear", slope="0.35", intercept="0.62")
    for el in list(root):
        if el.get("id") not in a.highlight:
            el.set("filter", "url(#faded)")
    root.insert(0, defs)


def read_alpha(png):
    """Minimal decoder for rsvg's 8-bit RGBA non-interlaced PNG output.
    Returns (w, h, list of rows of alpha values)."""
    data = open(png, "rb").read()
    pos, idat = 8, b""
    while pos < len(data):
        n, typ = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + n]
        if typ == b"IHDR":
            w, h, depth, ctype = struct.unpack(">IIBB", body[:10])
            assert depth == 8 and ctype == 6
        elif typ == b"IDAT":
            idat += body
        pos += 12 + n
    raw, stride = zlib.decompress(idat), 4 * w
    prev, rows = bytearray(stride), []
    for y in range(h):
        ft = raw[y * (stride + 1)]
        cur = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for x in range(stride):
            left = cur[x - 4] if x >= 4 else 0
            up = prev[x]
            if ft == 1: cur[x] = (cur[x] + left) & 255
            elif ft == 2: cur[x] = (cur[x] + up) & 255
            elif ft == 3: cur[x] = (cur[x] + ((left + up) >> 1)) & 255
            elif ft == 4:
                ul = prev[x - 4] if x >= 4 else 0
                pa, pb, pc = abs(up - ul), abs(left - ul), abs(left + up - 2 * ul)
                cur[x] = (cur[x] + (left if pa <= pb and pa <= pc else (up if pb <= pc else ul))) & 255
        rows.append(cur[3::4]); prev = cur
    return w, h, rows


# Measure the drawing's bounding box in viewBox units (1 px = 1 unit).
vb = [float(v) for v in root.get("viewBox").split()]
with tempfile.TemporaryDirectory() as td:
    svg, png = os.path.join(td, "m.svg"), os.path.join(td, "m.png")
    ET.ElementTree(root).write(svg)
    subprocess.run(["rsvg-convert", "-w", str(round(vb[2])), "-h", str(round(vb[3])), svg, "-o", png], check=True)
    w, h, rows = read_alpha(png)
ys = [y for y in range(h) if max(rows[y]) > 8]
xs = [x for x in range(w) if any(rows[y][x] > 8 for y in ys)]
x0, x1 = vb[0] + xs[0] * vb[2] / w, vb[0] + (xs[-1] + 1) * vb[2] / w
y0, y1 = vb[1] + ys[0] * vb[3] / h, vb[1] + (ys[-1] + 1) * vb[3] / h
cx, cy = (x0 + x1) / 2, (y0 + y1) / 2

# Smallest ASPECT canvas containing the drawing plus margin, centred on it.
bw, bh = (x1 - x0) * MARGIN, (y1 - y0) * MARGIN
cw = max(bw, bh * ASPECT); ch = cw / ASPECT
for attr in ("enable-background", "x", "y"):
    root.attrib.pop(attr, None)
root.set("viewBox", f"{cx - cw / 2:.2f} {cy - ch / 2:.2f} {cw:.2f} {ch:.2f}")
root.set("width", f"{cw:.0f}px"); root.set("height", f"{ch:.0f}px")
ET.ElementTree(root).write(a.dst, xml_declaration=True, encoding="utf-8")
print(f"{a.dst}: drawing {x1-x0:.0f}x{y1-y0:.0f}, canvas {cw:.0f}x{ch:.0f}")
