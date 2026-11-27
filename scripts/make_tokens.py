#!/usr/bin/env python3
"""Draw the board pieces: population tokens, cities and ships per faction.

    python3 scripts/make_tokens.py            # writes assets/textures/*.png
    python3 scripts/make_tokens.py --sheet    # also a contact sheet in target/

Every piece is an SVG rendered with `rsvg-convert`, so shapes and colours can
be tweaked here and regenerated.

- Token: a dark disc tinted toward the faction, a faction-coloured rim, and
  the faction's emblem in a light tint of its colour.
- City: a crenellated square filled with the faction colour and the emblem in
  dark -- nothing like a round token.
- Ship: a 2:1 rounded rectangle with a faction rim and a hull and sail.

Colours are the authentic ones from docs/ast.xslx (see ast_faction_color).
"""

import argparse
import os
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(REPO, "assets", "textures")

FACTIONS = {
    "africa": "#993300",
    "iberia": "#ff0000",
    "illyria": "#ffff00",
    "thrace": "#008000",
    "crete": "#99cc00",
    "asia": "#ff9900",
    "assyria": "#3366ff",
    "babylon": "#ccffff",
    "egypt": "#ffffcc",
}

# Emblems on a 100x100 box, centred at (50, 50). `{c}` is the ink colour.
EMBLEMS = {
    # Carthage: the sign of Tanit.
    "africa": """
        <circle cx="50" cy="24" r="11" fill="{c}"/>
        <rect x="18" y="38" width="64" height="9" rx="3" fill="{c}"/>
        <path d="M50 47 L80 86 L20 86 Z" fill="{c}"/>""",
    # A bull's head.
    "iberia": """
        <path d="M14 22 C14 44 28 46 38 42 L62 42 C72 46 86 44 86 22
                 C80 34 70 36 62 34 L38 34 C30 36 20 34 14 22 Z" fill="{c}"/>
        <path d="M34 40 L66 40 L62 72 C60 84 40 84 38 72 Z" fill="{c}"/>""",
    # The Illyrian sun.
    "illyria": """
        <circle cx="50" cy="50" r="16" fill="{c}"/>
        <g stroke="{c}" stroke-width="7" stroke-linecap="round">
          <line x1="50" y1="12" x2="50" y2="26"/><line x1="50" y1="74" x2="50" y2="88"/>
          <line x1="12" y1="50" x2="26" y2="50"/><line x1="74" y1="50" x2="88" y2="50"/>
          <line x1="23" y1="23" x2="33" y2="33"/><line x1="67" y1="67" x2="77" y2="77"/>
          <line x1="23" y1="77" x2="33" y2="67"/><line x1="67" y1="33" x2="77" y2="23"/>
        </g>""",
    # A peltast's crescent shield.
    "thrace": """
        <path d="M10 34 C22 92 78 92 90 34 C80 46 66 50 58 44 C56 40 54 36 50 32
                 C46 36 44 40 42 44 C34 50 20 46 10 34 Z" fill="{c}"/>""",
    # The double axe of Knossos.
    "crete": """
        <rect x="46" y="10" width="8" height="82" rx="3" fill="{c}"/>
        <path d="M46 24 C34 18 22 14 12 16 C18 30 18 44 12 58 C22 60 34 56 46 50 Z" fill="{c}"/>
        <path d="M54 24 C66 18 78 14 88 16 C82 30 82 44 88 58 C78 60 66 56 54 50 Z" fill="{c}"/>""",
    # The Trojan horse, on its wheeled platform.
    "asia": """
        <rect x="16" y="74" width="68" height="7" rx="2" fill="{c}"/>
        <circle cx="26" cy="86" r="7" fill="{c}"/><circle cx="74" cy="86" r="7" fill="{c}"/>
        <rect x="24" y="40" width="44" height="18" rx="7" fill="{c}"/>
        <g stroke="{c}" stroke-width="6"><line x1="30" y1="54" x2="30" y2="76"/>
          <line x1="40" y1="54" x2="40" y2="76"/><line x1="54" y1="54" x2="54" y2="76"/>
          <line x1="64" y1="54" x2="64" y2="76"/></g>
        <path d="M58 46 L70 16 L82 18 L90 30 L84 34 L76 28 L70 50 Z" fill="{c}"/>
        <path d="M70 16 L64 22 L62 34" stroke="{c}" stroke-width="4" fill="none"/>""",
    # A six-spoked chariot wheel.
    "assyria": """
        <circle cx="50" cy="50" r="36" fill="none" stroke="{c}" stroke-width="9"/>
        <circle cx="50" cy="50" r="9" fill="{c}"/>
        <g stroke="{c}" stroke-width="6">
          <line x1="50" y1="14" x2="50" y2="86"/>
          <line x1="19" y1="32" x2="81" y2="68"/>
          <line x1="19" y1="68" x2="81" y2="32"/>
        </g>""",
    # A ziggurat.
    "babylon": """
        <rect x="12" y="70" width="76" height="16" fill="{c}"/>
        <rect x="24" y="50" width="52" height="16" fill="{c}"/>
        <rect x="36" y="30" width="28" height="16" fill="{c}"/>
        <rect x="44" y="14" width="12" height="12" fill="{c}"/>""",
    # A pyramid with its courses.
    "egypt": """
        <path d="M50 12 L90 86 L10 86 Z" fill="{c}"/>
        <g stroke="#000" stroke-opacity="0.35" stroke-width="3">
          <line x1="34" y1="42" x2="66" y2="42"/>
          <line x1="25" y1="58" x2="75" y2="58"/>
          <line x1="17" y1="73" x2="83" y2="73"/>
        </g>""",
    # Pirates: crossed bones.
    "pirate": """
        <g stroke="{c}" stroke-width="10" stroke-linecap="round">
          <line x1="22" y1="22" x2="78" y2="78"/><line x1="78" y1="22" x2="22" y2="78"/>
        </g>""",
}


def hex_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i : i + 2], 16) for i in (0, 2, 4))


def mix(a, b, t):
    ra, rb = hex_rgb(a), hex_rgb(b)
    return "#%02x%02x%02x" % tuple(round(x + (y - x) * t) for x, y in zip(ra, rb))


def emblem(name, ink, x, y, size):
    return (
        f'<g transform="translate({x},{y}) scale({size / 100})">'
        + EMBLEMS[name].format(c=ink)
        + "</g>"
    )


def token_svg(name, color):
    w, h = 186, 184
    body = mix(color, "#101010", 0.62)
    ink = mix(color, "#ffffff", 0.45)
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
  <circle cx="93" cy="92" r="88" fill="#111"/>
  <circle cx="93" cy="92" r="85" fill="{color}"/>
  <circle cx="93" cy="92" r="77" fill="{body}"/>
  {emblem(name, ink, 32, 31, 122)}
</svg>"""


def city_svg(name, color, ink=None, wall="#111"):
    # A walled square: merlons along the top, a gate below the emblem.
    w, h = 186, 184
    ink = ink or mix(color, "#101010", 0.75)
    merlons = "".join(
        f'<rect x="{10 + i * 34}" y="6" width="20" height="26" fill="{color}" stroke="{wall}" stroke-width="5"/>'
        for i in range(5)
    )
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
  {merlons}
  <rect x="8" y="26" width="170" height="152" rx="6" fill="{color}" stroke="{wall}" stroke-width="7"/>
  <rect x="14" y="26" width="158" height="10" fill="{color}"/>
  {emblem(name, ink, 45, 42, 96)}
  <path d="M78 178 L78 150 C78 138 108 138 108 150 L108 178 Z" fill="{wall}"/>
</svg>"""


def ship_svg(color):
    w, h = 240, 120
    body = mix(color, "#101010", 0.62)
    ink = mix(color, "#ffffff", 0.45)
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
  <rect x="3" y="3" width="234" height="114" rx="22" fill="#111"/>
  <rect x="7" y="7" width="226" height="106" rx="19" fill="{color}"/>
  <rect x="15" y="15" width="210" height="90" rx="14" fill="{body}"/>
  <path d="M30 70 L210 70 C204 84 192 96 176 96 L64 96 C48 96 36 84 30 70 Z" fill="{ink}"/>
  <path d="M210 70 C222 60 222 46 212 40" stroke="{ink}" stroke-width="7" fill="none"/>
  <rect x="116" y="18" width="7" height="54" fill="{ink}"/>
  <path d="M84 24 L156 24 C162 40 162 52 156 64 L84 64 C78 52 78 40 84 24 Z" fill="{ink}"/>
  <g stroke="{body}" stroke-width="3"><line x1="98" y1="26" x2="98" y2="62"/>
    <line x1="142" y1="26" x2="142" y2="62"/></g>
</svg>"""


def render(svg, path):
    with tempfile.NamedTemporaryFile("w", suffix=".svg", delete=False) as f:
        f.write(svg)
        src = f.name
    try:
        subprocess.run(["rsvg-convert", src, "-o", path], check=True)
    finally:
        os.unlink(src)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--sheet", action="store_true", help="also write a contact sheet")
    args = parser.parse_args()

    written = []
    for name, color in FACTIONS.items():
        for suffix, svg in (
            ("", token_svg(name, color)),
            ("-city", city_svg(name, color)),
            ("-ship", ship_svg(color)),
        ):
            path = os.path.join(OUT, f"{name}{suffix}.png")
            render(svg, path)
            written.append(path)
    path = os.path.join(OUT, "pirate-city.png")
    render(city_svg("pirate", "#2b2b2b", ink="#e8e2d0", wall="#000"), path)
    written.append(path)
    print("\n".join(os.path.relpath(p, REPO) for p in written))

    if args.sheet:
        sheet = os.path.join(REPO, "target", "tokens-sheet.png")
        os.makedirs(os.path.dirname(sheet), exist_ok=True)
        rows = []
        for name in FACTIONS:
            rows.append(" ".join(os.path.join(OUT, f"{name}{s}.png") for s in ("", "-city", "-ship")))
        cmd = ["magick", "-background", "#c8b88a"]
        for name in FACTIONS:
            cmd += ["(", *[os.path.join(OUT, f"{name}{s}.png") for s in ("", "-city", "-ship")],
                    "-gravity", "center", "+append", ")"]
        cmd += ["-append", sheet]
        subprocess.run(cmd, check=True)
        print(os.path.relpath(sheet, REPO))


if __name__ == "__main__":
    sys.exit(main())
