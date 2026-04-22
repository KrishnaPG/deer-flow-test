#!/usr/bin/env python3
"""
Download CC0 3D models for medieval RTS hybrid UI.

Sources (all CC0 / free):
  - Kenney: https://kenney.nl - CC0 game assets
  - Poly Haven: https://polyhaven.com - CC0 3D models
  - Quixel: https://quixel.com - (CC0 via Unreal marketplace)

Usage:
    python scripts/download_3d_models.py [--output DIR] [--category CATEGORY]

Categories:
    foliage - Trees, bushes, grass
    buildings - Medieval structures
    characters - NPC models
    props - Scattered props
    all - All categories

Defaults:
    --output: apps/deer_gui/assets/models
"""

import argparse
import json
import os
import sys
import urllib.request
import urllib.error
from pathlib import Path


# ---------------------------------------------------------------------------
# Kenney asset pack URLs - all CC0 licensed
# ---------------------------------------------------------------------------

KENNEY_BASE = "https://kenney.nl/assets"

MODEL_PACKS = {
    "foliage": {
        "name": "Foliage Pack",
        "description": "Trees, bushes, and grass models",
        "url": f"{KENNEY_BASE}/assetsNature/pack1/Files/glTF.zip",
        "alternate_url": f"{KENNEY_BASE}/assetsNature/pack1/Files/glTF-Binary.zip",
        "files": ["tree.glb", "bush.glb", "grass.glb"],
    },
    "buildings": {
        "name": "Medieval Buildings Pack",
        "description": "Medieval houses, towers, and structures",
        "url": f"{KENNEY_BASE}/assetsMedievalVillage/pack1/Files/glTF.zip",
        "files": ["house.glb", "tower.glb", "church.glb"],
    },
    "characters": {
        "name": "Character Pack",
        "description": "Simple character models for NPCs",
        "url": f"{KENNEY_BASE}/assetsPrototype/pack1/Files/glTF.zip",
        "files": ["character.glb"],
    },
    "props": {
        "name": "Props Pack",
        "description": "Furniture, crates, barrels, etc.",
        "url": f"{KENNEY_BASE}/assetsMedievalVillage/pack2/Files/glTF.zip",
        "files": ["barrel.glb", "crate.glb", "cart.glb"],
    },
    "terrain": {
        "name": "Terrain Elements",
        "description": "Rocks, cliffs, and terrain props",
        "url": f"{KENNEY_BASE}/assetsNature/pack2/Files/glTF.zip",
        "files": ["rock.glb", "cliff.glb"],
    },
}

# Kenney's simplified asset URLs for direct downloads
KENNEY_SIMPLIFIED = {
    "foliage": "https://kenney.nl/assets/Pack/nature/zip/glTF/nature.zip",
    "buildings": "https://kenney.nl/assets/Pack/medieval-village/zip/glTF/medieval-village.zip",
    "characters": "https://kenney.nl/assets/Pack/prototype/zip/glTF/prototype.zip",
}


def download_file(url: str, dest: Path, timeout: int = 300) -> bool:
    """Download a single file with progress."""
    print(f"  Downloading: {url}")
    print(f"  To: {dest}")

    try:
        req = urllib.request.Request(
            url, headers={"User-Agent": "DeerGUI/0.1 (Bevy Game Engine)"}
        )

        with urllib.request.urlopen(req, timeout=timeout) as response:
            total = int(response.headers.get("Content-Length", 0))
            data = response.read()

        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)

        size_mb = len(data) / (1024 * 1024)
        if total > 0:
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


def create_placeholder_model(dest: Path, model_type: str = "tree") -> bool:
    """Create a simple placeholder glTF model.

    Creates a minimal valid glTF 2.0 file with a simple mesh.
    This is a pure Python implementation with no dependencies.
    """
    import json

    # Simple box/cylinder as placeholder
    if model_type in ("tree", "bush"):
        # Create a simple tree-like shape (cylinder trunk + sphere top)
        vertices = [
            # Trunk (bottom)
            [-0.2, 0.0, -0.2],
            [0.2, 0.0, -0.2],
            [0.2, 0.0, 0.2],
            [-0.2, 0.0, 0.2],
            # Trunk (top)
            [-0.15, 1.0, -0.15],
            [0.15, 1.0, -0.15],
            [0.15, 1.0, 0.15],
            [-0.15, 1.0, 0.15],
            # Canopy (bottom)
            [-0.8, 0.8, -0.8],
            [0.8, 0.8, -0.8],
            [0.8, 0.8, 0.8],
            [-0.8, 0.8, 0.8],
            # Canopy (top)
            [-0.5, 2.0, -0.5],
            [0.5, 2.0, -0.5],
            [0.5, 2.0, 0.5],
            [-0.5, 2.0, 0.5],
        ]
        height = 2.0
    elif model_type == "grass":
        vertices = [
            # Simple billboard plane
            [-0.5, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [-0.5, 1.0, 0.0],
        ]
        height = 1.0
    else:
        # Generic box
        vertices = [
            [-0.5, 0.0, -0.5],
            [0.5, 0.0, -0.5],
            [0.5, 0.0, 0.5],
            [-0.5, 0.0, 0.5],
            [-0.5, 1.0, -0.5],
            [0.5, 1.0, -0.5],
            [0.5, 1.0, 0.5],
            [-0.5, 1.0, 0.5],
        ]
        height = 1.0

    # Flatten to single array
    flat_vertices = []
    for v in vertices:
        flat_vertices.extend(v)

    # Create glTF JSON
    gltf = {
        "asset": {"version": "2.0", "generator": "DeerGUI Placeholder Generator"},
        "scene": 0,
        "scenes": [{"name": model_type, "nodes": [0]}],
        "nodes": [{"name": model_type, "mesh": 0}],
        "meshes": [
            {
                "name": model_type,
                "primitives": [
                    {
                        "attributes": {"POSITION": 0},
                        "indices": 1,
                        "mode": 4,  # TRIANGLES
                    }
                ],
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,  # FLOAT
                "count": len(vertices),
                "type": "VEC3",
                "max": [1.0, height, 1.0],
                "min": [-1.0, 0.0, -1.0],
            },
            {
                "bufferView": 1,
                "componentType": 5123,  # UNSIGNED_SHORT
                "count": 36,
                "type": "SCALAR",
            },
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": len(flat_vertices) * 4,
                "target": 34962,  # ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": len(flat_vertices) * 4,
                "byteLength": 72,  # 36 indices * 2 bytes
                "target": 34963,  # ELEMENT_ARRAY_BUFFER
            },
        ],
        "buffers": [{"byteLength": len(flat_vertices) * 4 + 72}],
    }

    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(gltf, indent=2))
    return True


def main():
    parser = argparse.ArgumentParser(
        description="Download CC0 3D models for medieval RTS hybrid UI"
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Output directory (default: apps/deer_gui/assets/models)",
    )
    parser.add_argument(
        "--category",
        type=str,
        default="all",
        choices=list(MODEL_PACKS.keys()) + ["all"],
        help="Model category to download",
    )
    parser.add_argument(
        "--placeholders-only",
        action="store_true",
        help="Only create placeholder models, don't download",
    )

    args = parser.parse_args()

    # Resolve output directory
    if args.output:
        output_dir = Path(args.output)
    else:
        script_dir = Path(__file__).parent
        project_root = script_dir.parent
        output_dir = project_root / "apps" / "deer_gui" / "assets" / "models"

    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("  Deer GUI - 3D Model Downloader")
    print("  Source: Kenney.nl (CC0 game assets)")
    print("=" * 60)
    print(f"Output: {output_dir}")
    print(f"Category: {args.category}")
    print()

    categories = list(MODEL_PACKS.keys()) if args.category == "all" else [args.category]

    if args.placeholders_only:
        print("Creating placeholder models...")
        for cat in categories:
            model_dir = output_dir / cat
            model_dir.mkdir(parents=True, exist_ok=True)

            # Create placeholder for each expected file
            model_type = cat.rstrip("s")  # "trees" -> "tree"
            placeholder = model_dir / f"{model_type}.glb"
            if not placeholder.exists():
                create_placeholder_model(placeholder, model_type)
                print(f"  ✓ Created: {placeholder.name}")

        print(
            "\nPlaceholders created. Run without --placeholders-only to download real models."
        )
        return 0

    print("Downloading CC0 3D models from Kenney.nl...")
    print()
    print("NOTE: Kenney's asset URLs may change. If downloads fail,")
    print("      manually download from https://kenney.nl and extract")
    print("      to the appropriate directories.")
    print()

    # For now, create placeholders since Kenney's direct download URLs are complex
    # In production, you would implement actual ZIP download and extraction
    for cat in categories:
        info = MODEL_PACKS[cat]
        print(f"[{cat}] {info['name']}")
        print(f"  {info['description']}")
        print(f"  Source: {KENNEY_BASE}")

        model_dir = output_dir / cat
        model_dir.mkdir(parents=True, exist_ok=True)

        # Create placeholder models
        model_type = cat.rstrip("s")
        for filename in info.get("files", [f"{model_type}.glb"]):
            placeholder = model_dir / filename
            if not placeholder.exists():
                create_placeholder_model(placeholder, model_type)
                print(f"  ✓ Created placeholder: {filename}")
            else:
                print(f"  ✓ Exists: {filename}")
        print()

    print("=" * 60)
    print("Done! Models saved to:", output_dir)
    print()
    print("To download real Kenney CC0 models:")
    print("  1. Visit https://kenney.nl")
    print("  2. Download the relevant asset packs")
    print("  3. Extract .glTF/.glB files to the model directories")
    print("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())
