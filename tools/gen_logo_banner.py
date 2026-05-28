"""Gerador de logo banner first-plan em estilo SNES pixel art.

Output: assets/logo-fp-banner.svg

Layers (back to front):
1. Background gradient (cobalt -> deep purple)
2. Scanlines horizontais a cada 4px
3. Drop shadow vermelho (+6, +6) das letras
4. Outline preto (+1px todos lados) das letras
5. Fill dourado das letras
6. Highlight creme no topo de cada pixel exposto
"""

from pathlib import Path

# Paleta SNES Mario All-Stars
COLOR_BG_TOP = "#1E1B4B"      # navy escuro
COLOR_BG_BOTTOM = "#4C1D95"   # roxo profundo
COLOR_SCANLINE = "#000000"
COLOR_SHADOW = "#DC2626"
COLOR_OUTLINE = "#000000"
COLOR_FILL = "#FCD34D"
COLOR_HIGHLIGHT = "#FEF3C7"

# Bitmap glyphs 5x7 (# = filled, . = empty)
GLYPHS = {
    "F": [
        "#####",
        "#....",
        "#....",
        "####.",
        "#....",
        "#....",
        "#....",
    ],
    "I": [
        "#####",
        "..#..",
        "..#..",
        "..#..",
        "..#..",
        "..#..",
        "#####",
    ],
    "R": [
        "####.",
        "#...#",
        "#...#",
        "####.",
        "#.#..",
        "#..#.",
        "#...#",
    ],
    "S": [
        ".####",
        "#....",
        "#....",
        ".###.",
        "....#",
        "....#",
        "####.",
    ],
    "T": [
        "#####",
        "..#..",
        "..#..",
        "..#..",
        "..#..",
        "..#..",
        "..#..",
    ],
    "-": [
        ".....",
        ".....",
        ".....",
        ".###.",
        ".....",
        ".....",
        ".....",
    ],
    "P": [
        "####.",
        "#...#",
        "#...#",
        "####.",
        "#....",
        "#....",
        "#....",
    ],
    "L": [
        "#....",
        "#....",
        "#....",
        "#....",
        "#....",
        "#....",
        "#####",
    ],
    "A": [
        ".###.",
        "#...#",
        "#...#",
        "#####",
        "#...#",
        "#...#",
        "#...#",
    ],
    "N": [
        "#...#",
        "##..#",
        "#.#.#",
        "#.#.#",
        "#.#.#",
        "#..##",
        "#...#",
    ],
}

TEXT = "FIRST-PLAN"
SCALE = 10              # px svg per "pixel" do glyph
GLYPH_W = 5 * SCALE     # 50
GLYPH_H = 7 * SCALE     # 70
GLYPH_GAP = SCALE       # 10
PAD_X = 24
PAD_Y = 24
SHADOW_OFFSET = 6


def filled_cells(glyph: list[str]) -> list[tuple[int, int]]:
    out = []
    for row, line in enumerate(glyph):
        for col, ch in enumerate(line):
            if ch == "#":
                out.append((col, row))
    return out


def is_top_exposed(glyph: list[str], col: int, row: int) -> bool:
    """Highlight aplica-se a pixels cujo vizinho acima e empty (ou fora do glyph)."""
    if row == 0:
        return True
    return glyph[row - 1][col] != "#"


def main() -> None:
    # Total width: N letters * GLYPH_W + (N-1) * GLYPH_GAP + 2*PAD_X
    text_width = len(TEXT) * GLYPH_W + (len(TEXT) - 1) * GLYPH_GAP
    width = text_width + 2 * PAD_X
    height = GLYPH_H + 2 * PAD_Y

    rects = []

    # --- Layer 1: background gradient ---
    defs = [
        '<defs>',
        '  <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">',
        f'    <stop offset="0%" stop-color="{COLOR_BG_TOP}"/>',
        f'    <stop offset="100%" stop-color="{COLOR_BG_BOTTOM}"/>',
        '  </linearGradient>',
        '  <radialGradient id="vignette" cx="0.5" cy="0.5" r="0.7">',
        '    <stop offset="60%" stop-color="#000000" stop-opacity="0"/>',
        '    <stop offset="100%" stop-color="#000000" stop-opacity="0.6"/>',
        '  </radialGradient>',
        '</defs>',
    ]

    bg_rect = f'<rect x="0" y="0" width="{width}" height="{height}" fill="url(#bg)"/>'

    # --- Layer 2: scanlines (a cada 4 svg units, opacity 0.15) ---
    scanlines = []
    y = 0
    while y < height:
        scanlines.append(
            f'<rect x="0" y="{y}" width="{width}" height="1" fill="{COLOR_SCANLINE}" opacity="0.15"/>'
        )
        y += 4

    # --- Layers 3-6: letras ---
    shadow_rects = []
    outline_rects = []
    fill_rects = []
    highlight_rects = []

    x_cursor = PAD_X
    for ch in TEXT:
        glyph = GLYPHS.get(ch)
        if glyph is None:
            raise ValueError(f"Glifo nao definido: {ch!r}")
        cells = filled_cells(glyph)

        for col, row in cells:
            px = x_cursor + col * SCALE
            py = PAD_Y + row * SCALE

            # Shadow: offset +SHADOW_OFFSET
            shadow_rects.append(
                f'<rect x="{px + SHADOW_OFFSET - 1}" y="{py + SHADOW_OFFSET - 1}" '
                f'width="{SCALE + 2}" height="{SCALE + 2}" fill="{COLOR_SHADOW}"/>'
            )
            # Outline: +1 each side
            outline_rects.append(
                f'<rect x="{px - 1}" y="{py - 1}" '
                f'width="{SCALE + 2}" height="{SCALE + 2}" fill="{COLOR_OUTLINE}"/>'
            )
            # Fill
            fill_rects.append(
                f'<rect x="{px}" y="{py}" width="{SCALE}" height="{SCALE}" fill="{COLOR_FILL}"/>'
            )
            # Highlight top 30% se top exposto
            if is_top_exposed(glyph, col, row):
                h_height = max(2, SCALE // 3)
                highlight_rects.append(
                    f'<rect x="{px}" y="{py}" width="{SCALE}" height="{h_height}" '
                    f'fill="{COLOR_HIGHLIGHT}"/>'
                )

        x_cursor += GLYPH_W + GLYPH_GAP

    # --- Layer final: vignette overlay ---
    vignette = f'<rect x="0" y="0" width="{width}" height="{height}" fill="url(#vignette)"/>'

    # Compose SVG
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" shape-rendering="crispEdges">',
        '<!-- first-plan banner logo, SNES Mario All-Stars palette, retro CRT background -->',
        *defs,
        bg_rect,
        *scanlines,
        *shadow_rects,
        *outline_rects,
        *fill_rects,
        *highlight_rects,
        vignette,
        '</svg>',
    ]

    output = "\n".join(parts) + "\n"
    out_path = Path(__file__).resolve().parent.parent / "assets" / "logo-fp-banner.svg"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(output)
    print(f"wrote {out_path}")
    print(f"  viewBox: 0 0 {width} {height}")
    print(f"  rects: bg=1 + scanlines={len(scanlines)} + shadow={len(shadow_rects)} + outline={len(outline_rects)} + fill={len(fill_rects)} + highlight={len(highlight_rects)} + vignette=1")
    print(f"  total: {1 + len(scanlines) + len(shadow_rects) + len(outline_rects) + len(fill_rects) + len(highlight_rects) + 1}")


if __name__ == "__main__":
    main()
