#!/usr/bin/env python3
"""Color-key sprite sheets: replace a background color with transparency.

By default, the key color is auto-detected from the top-left pixel of each
image. Override with --color RRGGBB. A tolerance (--tolerance, default 0)
allows near-matches to also become transparent.

Usage:
    python tools/color_key.py assets/raw/hero_sheet.bmp
    python tools/color_key.py assets/raw/hero_sheet.bmp -o assets/textures/hero_sheet.png
    python tools/color_key.py assets/raw/spritesheets/ -o assets/textures/
    python tools/color_key.py sheet.png --color FF00FF --tolerance 10
    python tools/color_key.py folder/ --dry-run
"""

import argparse
import os
import sys
from pathlib import Path

from PIL import Image

SUPPORTED_EXTENSIONS = {".png", ".bmp", ".gif", ".jpg", ".jpeg", ".tga", ".tiff", ".tif"}


def parse_hex_color(hex_str: str) -> tuple[int, int, int]:
    """Parse a hex color string (with or without #) into (R, G, B)."""
    hex_str = hex_str.lstrip("#")
    if len(hex_str) != 6:
        raise ValueError(f"Expected 6-character hex color, got '{hex_str}'")
    return (
        int(hex_str[0:2], 16),
        int(hex_str[2:4], 16),
        int(hex_str[4:6], 16),
    )


def detect_key_color(img: Image.Image) -> tuple[int, int, int]:
    """Sample the top-left pixel as the key color."""
    pixel = img.getpixel((0, 0))
    if isinstance(pixel, int):
        # Grayscale
        return (pixel, pixel, pixel)
    return (pixel[0], pixel[1], pixel[2])


def color_distance(c1: tuple[int, int, int], c2: tuple[int, int, int]) -> int:
    """Manhattan distance between two RGB colors."""
    return abs(c1[0] - c2[0]) + abs(c1[1] - c2[1]) + abs(c1[2] - c2[2])


def color_key_image(
    img: Image.Image,
    key_color: tuple[int, int, int] | None,
    tolerance: int,
) -> tuple[Image.Image, tuple[int, int, int], int]:
    """Replace pixels matching key_color (within tolerance) with transparent.

    Returns (result_image, detected_key_color, pixel_count_replaced).
    """
    img = img.convert("RGBA")

    if key_color is None:
        key_color = detect_key_color(img)

    pixels = img.load()
    width, height = img.size
    replaced = 0

    for y in range(height):
        for x in range(width):
            r, g, b, a = pixels[x, y]
            if color_distance((r, g, b), key_color) <= tolerance:
                pixels[x, y] = (0, 0, 0, 0)
                replaced += 1

    return img, key_color, replaced


def resolve_output_path(input_path: Path, output: Path | None, is_batch: bool) -> Path:
    """Determine the output file path for a given input file."""
    if output is None:
        # Default: same directory, same stem, .png extension
        return input_path.with_suffix(".png")

    if is_batch:
        # output is a directory
        return output / (input_path.stem + ".png")

    # output is a file path
    return output


def collect_input_files(input_path: Path) -> list[Path]:
    """Collect image files from a path (single file or directory)."""
    if input_path.is_file():
        if input_path.suffix.lower() in SUPPORTED_EXTENSIONS:
            return [input_path]
        print(f"Warning: '{input_path}' is not a supported image format", file=sys.stderr)
        return []

    if input_path.is_dir():
        files = []
        for child in sorted(input_path.iterdir()):
            if child.is_file() and child.suffix.lower() in SUPPORTED_EXTENSIONS:
                files.append(child)
        return files

    print(f"Error: '{input_path}' does not exist", file=sys.stderr)
    return []


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Color-key sprite sheets: replace a background color with transparency.",
        epilog="Supported formats: " + ", ".join(sorted(SUPPORTED_EXTENSIONS)),
    )
    parser.add_argument(
        "input",
        type=Path,
        help="Input image file or directory of images",
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        default=None,
        help="Output file (single input) or directory (batch input). Default: same location as input with .png extension.",
    )
    parser.add_argument(
        "--color",
        type=str,
        default=None,
        help="Key color as hex (e.g. FF00FF). Default: auto-detect from top-left pixel of each image.",
    )
    parser.add_argument(
        "--tolerance",
        type=int,
        default=0,
        help="Color matching tolerance (Manhattan distance in RGB space, 0-765). Default: 0 (exact match).",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be processed without writing files.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite existing output files without prompting.",
    )

    args = parser.parse_args()

    # Parse fixed key color if provided
    fixed_key = None
    if args.color is not None:
        try:
            fixed_key = parse_hex_color(args.color)
        except ValueError as e:
            print(f"Error: {e}", file=sys.stderr)
            return 1

    if args.tolerance < 0 or args.tolerance > 765:
        print("Error: --tolerance must be between 0 and 765", file=sys.stderr)
        return 1

    # Collect input files
    input_files = collect_input_files(args.input)
    if not input_files:
        print("No supported image files found.", file=sys.stderr)
        return 1

    is_batch = args.input.is_dir()

    # Validate output for batch mode
    if is_batch and args.output is not None:
        if args.output.exists() and not args.output.is_dir():
            print(f"Error: batch output '{args.output}' must be a directory", file=sys.stderr)
            return 1
        if not args.output.exists():
            if not args.dry_run:
                args.output.mkdir(parents=True, exist_ok=True)

    print(f"Processing {len(input_files)} file(s)...")
    if fixed_key:
        print(f"Key color: #{fixed_key[0]:02X}{fixed_key[1]:02X}{fixed_key[2]:02X}")
    else:
        print("Key color: auto-detect (top-left pixel)")
    if args.tolerance > 0:
        print(f"Tolerance: {args.tolerance}")
    print()

    processed = 0
    skipped = 0

    for input_file in input_files:
        output_path = resolve_output_path(input_file, args.output, is_batch)

        if output_path.exists() and not args.overwrite:
            # Don't overwrite the input file if output would be the same path
            # and input is already a PNG (unless --overwrite is set)
            if output_path == input_file and input_file.suffix.lower() == ".png":
                pass  # Allow in-place processing of PNGs
            elif output_path.exists():
                print(f"  SKIP {input_file.name} -> {output_path} (exists, use --overwrite)")
                skipped += 1
                continue

        if args.dry_run:
            key_label = f"#{fixed_key[0]:02X}{fixed_key[1]:02X}{fixed_key[2]:02X}" if fixed_key else "auto"
            print(f"  [dry-run] {input_file.name} -> {output_path} (key: {key_label})")
            processed += 1
            continue

        try:
            img = Image.open(input_file)
        except Exception as e:
            print(f"  ERROR {input_file.name}: failed to open: {e}", file=sys.stderr)
            skipped += 1
            continue

        result, detected_key, replaced = color_key_image(img, fixed_key, args.tolerance)

        total_pixels = result.size[0] * result.size[1]
        pct = (replaced / total_pixels * 100) if total_pixels > 0 else 0

        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)

        result.save(output_path, "PNG")
        key_hex = f"#{detected_key[0]:02X}{detected_key[1]:02X}{detected_key[2]:02X}"
        print(
            f"  OK {input_file.name} -> {output_path.name}  "
            f"key={key_hex}  {replaced}/{total_pixels} pixels ({pct:.1f}%) "
            f"[{result.size[0]}x{result.size[1]}]"
        )
        processed += 1

    print()
    print(f"Done: {processed} processed, {skipped} skipped")
    return 0


if __name__ == "__main__":
    sys.exit(main())
