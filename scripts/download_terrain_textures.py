#!/usr/bin/env python3
"""
Download CC0 high-quality PBR terrain textures for photo-realistic medieval terrain.

Sources (all CC0):
  - AmbientCG: https://ambientcg.com - PBR textures
  - Poly Haven: https://polyhaven.com - HDRIs and textures
  - Quixel: https://quixel.com - (CC0 via Unreal marketplace)

Usage:
    python scripts/download_terrain_textures.py [--output DIR] [--resolution RES]
    python scripts/download_terrain_textures.py --placeholders-only

Defaults:
    --output: apps/deer_gui/assets/textures/terrain
    --resolution: 2K (options: 512, 1K, 2K, 4K, 8K)
"""

import argparse
import json
import os
import sys
import urllib.request
import urllib.error
from pathlib import Path
from typing import Optional


# ---------------------------------------------------------------------------
# CC0 texture sources - photo-realistic terrain textures
# ---------------------------------------------------------------------------
# These are high-quality CC0 textures suitable for photo-realistic rendering.
# Each entry maps to AmbientCG's texture IDs which provide full PBR sets
# (Color, Normal, Roughness, Displacement, AO).

TERRAIN_TEXTURES = {
    "grass": {
        "name": "Meadow Grass",
        "ambientcg_id": "Grass003",
        "description": "Lush green meadow grass for valleys and plains",
        "maps": ["Color", "NormalGL", "Roughness", "Displacement", "AmbientOcclusion"],
    },
    "dirt": {
        "name": "Forest Dirt",
        "ambientcg_id": "Ground011",
        "description": "Rich brown forest floor dirt",
        "maps": ["Color", "NormalGL", "Roughness", "Displacement", "AmbientOcclusion"],
    },
    "rock": {
        "name": "Cliff Rock",
        "ambientcg_id": "Rock030",
        "description": "Weathered grey rock for mountains",
        "maps": ["Color", "NormalGL", "Roughness", "Displacement", "AmbientOcclusion"],
    },
    "snow": {
        "name": "Fresh Snow",
        "ambientcg_id": "Snow006",
        "description": "Bright white snow for winter terrain",
        "maps": ["Color", "NormalGL", "Roughness", "Displacement"],
    },
}

# Alternate textures for variety
VARIATION_TEXTURES = {
    "grass_variant": "Grass014",
    "dirt_variant": "Ground026",
    "rock_variant": "Rock032",
    "cliff": "Cliff001",
    "moss": "Moss001",
    "gravel": "Gravel003",
    "sand": "Sand001",
    "mud": "Ground026",
}

AMBIENTCG_DOWNLOAD_BASE = "https://dl.ambientcg.com"


def download_with_progress(url: str, dest: Path, timeout: int = 120) -> bool:
    """Download a file with progress indication."""
    print(f"  → {url}")

    try:
        req = urllib.request.Request(
            url, headers={"User-Agent": "DeerGUI/0.1 (Bevy Game Engine)"}
        )

        with urllib.request.urlopen(req, timeout=timeout) as response:
            total_size = int(response.headers.get("Content-Length", 0))
            data = response.read()

        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)

        size_mb = len(data) / (1024 * 1024)
        if total_size > 0:
            print(f"  ✓ Downloaded {size_mb:.1f} MB")
        else:
            print(f"  ✓ Saved ({size_mb:.1f} MB)")
        return True

    except urllib.error.HTTPError as e:
        print(f"  ✗ HTTP {e.code}: {e.reason}")
        return False
    except urllib.error.URLError as e:
        print(f"  ✗ URL Error: {e.reason}")
        return False
    except Exception as e:
        print(f"  ✗ Error: {e}")
        return False


def download_ambientcg_texture(
    texture_id: str,
    output_dir: Path,
    resolution: str = "2K",
    maps: Optional[list[str]] = None,
) -> tuple[int, int]:
    """Download a CC0 PBR texture set from AmbientCG.

    Returns: (success_count, total_count)
    """
    if maps is None:
        maps = ["Color", "NormalGL", "Roughness"]

    texture_dir = output_dir / texture_id.lower()
    texture_dir.mkdir(parents=True, exist_ok=True)

    success = 0
    total = len(maps)

    for map_name in maps:
        # AmbientCG URL format: /texid/texid_MapName_Resolution.ext
        # Color is PNG, others are typically JPG
        ext = "png" if map_name == "Color" else "jpg"
        filename = f"{texture_id}_{map_name}_{resolution}.{ext}"
        url = f"{AMBIENTCG_DOWNLOAD_BASE}/{texture_id}/{filename}"
        dest = texture_dir / filename

        if dest.exists() and dest.stat().st_size > 1000:
            print(f"  ✓ Exists: {dest.name}")
            success += 1
            continue

        if download_with_progress(url, dest):
            success += 1
        else:
            # Try alternate resolution format
            alt_ext = "jpg" if ext == "png" else "png"
            alt_filename = f"{texture_id}_{map_name}_{resolution}.{alt_ext}"
            alt_dest = texture_dir / alt_filename
            if download_with_progress(url.replace(f".{ext}", f".{alt_ext}"), alt_dest):
                success += 1

    return success, total


def create_procedural_texture(
    dest: Path, size: int = 512, texture_type: str = "grass"
) -> bool:
    """Create a procedural placeholder texture (no dependencies required)."""
    import struct
    import zlib

    # Base colors for different terrain types
    color_ranges = {
        "grass": ((60, 140, 40), (90, 170, 60)),
        "dirt": ((90, 60, 30), (130, 90, 50)),
        "rock": ((100, 100, 100), (140, 135, 130)),
        "snow": ((220, 225, 230), (240, 245, 250)),
    }

    min_rgb, max_rgb = color_ranges.get(texture_type, color_ranges["grass"])

    def make_png(w: int, h: int, pixels: list) -> bytes:
        """Create PNG from RGB pixel data."""

        def chunk(ctype: bytes, data: bytes) -> bytes:
            c = ctype + data
            return (
                struct.pack(">I", len(data))
                + c
                + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
            )

        sig = b"\x89PNG\r\n\x1a\n"
        ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))

        raw = b""
        for row in pixels:
            raw += b"\x00" + bytes(row)

        idat = chunk(b"IDAT", zlib.compress(raw))
        iend = chunk(b"IEND", b"")

        return sig + ihdr + idat + iend

    # Generate texture with simple noise pattern
    import random

    random.seed(hash(texture_type))  # Deterministic per type

    pixels = []
    for y in range(size):
        row = []
        for x in range(size):
            # Simple value noise
            nx, ny = x / size, y / size
            n = random.random() * 0.3

            r = int(min_rgb[0] + (max_rgb[0] - min_rgb[0]) * n)
            g = int(min_rgb[1] + (max_rgb[1] - min_rgb[1]) * n)
            b = int(min_rgb[2] + (max_rgb[2] - min_rgb[2]) * n)

            row.extend([min(r, 255), min(g, 255), min(b, 255)])
        pixels.append(row)

    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(make_png(size, size, pixels))
    return True


def create_placeholder_set(output_dir: Path, texture_type: str, size: int = 512) -> int:
    """Create a full set of placeholder textures for a terrain type."""
    texture_dir = output_dir / texture_type
    texture_dir.mkdir(parents=True, exist_ok=True)

    created = 0

    # Color texture
    color_path = texture_dir / f"{texture_type}_Color_{size}x{size}.png"
    if not color_path.exists():
        create_procedural_texture(color_path, size, texture_type)
        created += 1
        print(f"  ✓ Created placeholder: {color_path.name}")

    # Normal map (flat - neutral blue)
    normal_path = texture_dir / f"{texture_type}_NormalGL_{size}x{size}.png"
    if not normal_path.exists():
        create_flat_normal(normal_path, size)
        created += 1
        print(f"  ✓ Created placeholder: {normal_path.name}")

    # Roughness (medium grey)
    rough_path = texture_dir / f"{texture_type}_Roughness_{size}x{size}.png"
    if not rough_path.exists():
        create_flat_texture(rough_path, size, (180, 180, 180))
        created += 1
        print(f"  ✓ Created placeholder: {rough_path.name}")

    return created


def create_flat_normal(dest: Path, size: int) -> None:
    """Create a flat (neutral) normal map."""
    import struct
    import zlib

    def make_png(w: int, h: int, pixels: list) -> bytes:
        def chunk(ctype: bytes, data: bytes) -> bytes:
            c = ctype + data
            return (
                struct.pack(">I", len(data))
                + c
                + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
            )

        sig = b"\x89PNG\r\n\x1a\n"
        ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        raw = b""
        for row in pixels:
            raw += b"\x00" + bytes(row)
        idat = chunk(b"IDAT", zlib.compress(raw))
        iend = chunk(b"IEND", b"")
        return sig + ihdr + idat + iend

    # Neutral normal = (128, 128, 255) = pointing straight up
    pixels = [[128, 128, 255] * size for _ in range(size)]
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(make_png(size, size, pixels))


def create_flat_texture(dest: Path, size: int, color: tuple) -> None:
    """Create a flat single-color texture."""
    import struct
    import zlib

    def make_png(w: int, h: int, pixels: list) -> bytes:
        def chunk(ctype: bytes, data: bytes) -> bytes:
            c = ctype + data
            return (
                struct.pack(">I", len(data))
                + c
                + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
            )

        sig = b"\x89PNG\r\n\x1a\n"
        ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        raw = b""
        for row in pixels:
            raw += b"\x00" + bytes(row)
        idat = chunk(b"IDAT", zlib.compress(raw))
        iend = chunk(b"IEND", b"")
        return sig + ihdr + idat + iend

    pixels = [[color[0], color[1], color[2]] * size for _ in range(size)]
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(make_png(size, size, pixels))


def main():
    parser = argparse.ArgumentParser(
        description="Download CC0 terrain textures for photo-realistic medieval terrain"
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Output directory (default: apps/deer_gui/assets/textures/terrain)",
    )
    parser.add_argument(
        "--resolution",
        type=str,
        default="2K",
        choices=["512", "1K", "2K", "4K", "8K"],
        help="Texture resolution (default: 2K)",
    )
    parser.add_argument(
        "--placeholders-only",
        action="store_true",
        help="Only create procedural placeholders",
    )
    parser.add_argument(
        "--texture",
        type=str,
        default=None,
        choices=list(TERRAIN_TEXTURES.keys()),
        help="Download specific texture only",
    )

    args = parser.parse_args()

    # Resolve output directory
    if args.output:
        output_dir = Path(args.output)
    else:
        script_dir = Path(__file__).parent
        output_dir = (
            script_dir.parent / "apps" / "deer_gui" / "assets" / "textures" / "terrain"
        )

    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("  Deer GUI - Terrain Texture Downloader")
    print("  Source: AmbientCG (CC0 PBR textures)")
    print("=" * 60)
    print(f"Output: {output_dir}")
    print(f"Resolution: {args.resolution}")
    print()

    # Placeholder mode
    if args.placeholders_only:
        print("Creating procedural placeholder textures...")
        total = 0
        for tex_id in TERRAIN_TEXTURES:
            total += create_placeholder_set(output_dir, tex_id)
        print(f"\n✓ Created {total} placeholder textures")
        print("Run without --placeholders-only to download real CC0 textures.")
        return 0

    # Download mode
    print(f"Downloading {len(TERRAIN_TEXTURES)} CC0 texture sets...\n")

    downloaded = 0
    failed = 0

    for i, (tex_id, info) in enumerate(TERRAIN_TEXTURES.items(), 1):
        if args.texture and args.texture != tex_id:
            continue

        print(f"[{i}/{len(TERRAIN_TEXTURES)}] {info['name']}")
        print(f"    {info['description']}")

        success, total = download_ambientcg_texture(
            info["ambientcg_id"], output_dir, args.resolution, info["maps"]
        )

        if success == total:
            downloaded += 1
            print(f"  ✓ Complete\n")
        elif success > 0:
            print(f"  ⚠ Partial ({success}/{total})")
            print(f"  Creating placeholders for missing maps...\n")
            create_placeholder_set(output_dir, tex_id)
            failed += 1
        else:
            print(f"  ✗ Failed, creating placeholders...\n")
            create_placeholder_set(output_dir, tex_id)
            failed += 1

    print("=" * 60)
    print(f"Results: {downloaded} downloaded, {failed} with placeholders")
    print(f"Assets: {output_dir}")
    print("=" * 60)

    if failed > 0:
        print("\nNote: Placeholder textures were created for failed downloads.")
        print("These work for development but download real textures for production.")

    return 0 if downloaded == len(TERRAIN_TEXTURES) else 1


if __name__ == "__main__":
    sys.exit(main())
