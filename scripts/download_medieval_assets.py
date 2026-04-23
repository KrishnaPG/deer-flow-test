#!/usr/bin/env python3
"""
Download CC0 medieval 3D assets for the hybrid RTS/FPS scene.

Sources:
  - Kenney.nl: CC0 game assets (trees, rocks, buildings)
  - Quaternius: CC0 3D models (characters, animals)
  
Usage:
    python scripts/download_medieval_assets.py [--output DIR]
"""

import argparse
import os
import sys
import urllib.request
import zipfile
import tempfile
from pathlib import Path

# Kenney.nl CC0 asset packs
KENNEY_ASSETS = {
    "nature_kit": {
        "name": "Nature Kit",
        "url": "https://kenney.nl/media/pages/assets/nature-kit/81d707162c-1716922049/kenney_nature-kit.zip",
        "description": "Trees, rocks, bushes, grass",
    },
    "medieval_village": {
        "name": "Medieval Village", 
        "url": "https://kenney.nl/media/pages/assets/medieval-village/6e13a15b5e-1716922050/kenney_medieval-village.zip",
        "description": "Houses, walls, towers, props",
    },
}

def download_file(url: str, dest: Path, timeout: int = 120) -> bool:
    print(f"  Downloading: {url}")
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "DeerGUI/0.1"})
        with urllib.request.urlopen(req, timeout=timeout) as response:
            data = response.read()
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)
        size_mb = len(data) / (1024 * 1024)
        print(f"  ✓ Downloaded {size_mb:.1f} MB")
        return True
    except Exception as e:
        print(f"  ✗ Failed: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description="Download CC0 medieval assets")
    parser.add_argument("--output", type=str, default="apps/deer_gui/assets")
    args = parser.parse_args()
    
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print("=" * 60)
    print("  Medieval Asset Downloader")
    print("=" * 60)
    
    for key, info in KENNEY_ASSETS.items():
        print(f"\n[{key}] {info['name']}")
        
        with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as tmp:
            tmp_path = Path(tmp.name)
            if download_file(info["url"], tmp_path):
                with zipfile.ZipFile(tmp_path, 'r') as z:
                    z.extractall(output_dir / "models" / "kenney" / key)
                print(f"  ✓ Extracted to models/kenney/{key}")
            tmp_path.unlink(missing_ok=True)
    
    print("\n" + "=" * 60)
    print(f"Done! Assets in: {output_dir}")
    print("=" * 60)

if __name__ == "__main__":
    sys.exit(main() or 0)
