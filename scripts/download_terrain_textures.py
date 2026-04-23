#!/usr/bin/env python3
"""
Download CC0 high-quality PBR terrain textures for photo-realistic medieval terrain.

Sources (all CC0):
  - AmbientCG: https://ambientcg.com - PBR textures

Usage:
    python scripts/download_terrain_textures.py [--output DIR] [--resolution RES]
    python scripts/download_terrain_textures.py --texture grass

Defaults:
    --output: apps/deer_gui/assets/textures/terrain
    --resolution: 1K (options: 1K, 2K, 4K, 8K)
"""

import argparse
import json
import os
import sys
import urllib.request
import urllib.error
import zipfile
import tempfile
import re
from pathlib import Path

TERRAIN_TEXTURES = {
    "grass": {
        "name": "Meadow Grass",
        "ambientcg_id": "Grass003",
        "description": "Lush green meadow grass for valleys and plains",
    },
    "dirt": {
        "name": "Forest Dirt",
        "ambientcg_id": "Ground011",
        "description": "Rich brown forest floor dirt",
    },
    "rock": {
        "name": "Cliff Rock",
        "ambientcg_id": "Rock030",
        "description": "Weathered grey rock for mountains",
    },
    "snow": {
        "name": "Fresh Snow",
        "ambientcg_id": "Snow006",
        "description": "Bright white snow for winter terrain",
    },
}

def update_scene_configs(project_root: Path, texture_key: str, new_color_path: str):
    """Update all .scene.ron files to point to the newly downloaded texture."""
    scenes_dir = project_root / "apps" / "deer_gui" / "assets" / "scenes"
    if not scenes_dir.exists():
        return

    # Look for the old pattern, which could be the procedural name or an older resolution of the same ambientcg_id
    # We'll just replace lines containing `textures/terrain/{texture_key}/` OR `textures/terrain/{ambientcg_id}/` 
    # but we need to be careful to only replace the ones used in `layer_textures`.
    # A safer regex: find `"textures/terrain/.*?/.*?"` and if it corresponds to the current category, replace it.
    # Actually, the user specifically mentioned we should update the config for the respective resolution.
    
    # We will search for any string matching "textures/terrain/KEY/.*_Color.*\.(png|jpg)" 
    # and replace it with the new color path.
    
    pattern = re.compile(r'"textures/terrain/' + texture_key + r'/[^"]*?(Color|color)[^"]*?\.(png|jpg)"')
    
    for ron_file in scenes_dir.glob("*.scene.ron"):
        content = ron_file.read_text()
        if pattern.search(content):
            new_content = pattern.sub(f'"{new_color_path}"', content)
            if new_content != content:
                ron_file.write_text(new_content)
                print(f"  ✓ Updated config: {ron_file.name}")


def download_ambientcg_texture(
    texture_key: str,
    texture_id: str,
    output_dir: Path,
    resolution: str = "1K",
    format_pref: str = "PNG"
) -> bool:
    """Download a CC0 PBR texture set from AmbientCG using v3 API."""
    
    # Ensure texture specific directory exists
    texture_dir = output_dir / texture_key
    texture_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"  Querying AmbientCG API for {texture_id}...")
    api_url = f"https://ambientcg.com/api/v3/assets?id={texture_id}&include=downloads"
    
    try:
        req = urllib.request.Request(api_url, headers={"User-Agent": "DeerGUI/0.1"})
        with urllib.request.urlopen(req, timeout=15) as response:
            data = json.loads(response.read().decode("utf-8"))
    except Exception as e:
        print(f"  ✗ Failed to fetch API metadata: {e}")
        return False
        
    if not data.get("assets"):
        print(f"  ✗ Asset {texture_id} not found in AmbientCG.")
        return False
        
    asset_info = data["assets"][0]
    downloads = asset_info.get("downloads", [])
    
    # Find the matching download link
    target_attr = f"{resolution}-{format_pref}"
    download_url = None
    
    for dl in downloads:
        if dl.get("attributes") == target_attr and dl.get("extension") == "zip":
            download_url = dl.get("url")
            break
            
    # Fallback to JPG if PNG is not available
    if not download_url and format_pref == "PNG":
        target_attr = f"{resolution}-JPG"
        for dl in downloads:
            if dl.get("attributes") == target_attr and dl.get("extension") == "zip":
                download_url = dl.get("url")
                break
                
    if not download_url:
        print(f"  ✗ Could not find a ZIP download for {resolution} resolution.")
        return False
        
    print(f"  Downloading ZIP: {download_url}")
    try:
        with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as tmp:
            tmp_path = Path(tmp.name)
            req = urllib.request.Request(download_url, headers={"User-Agent": "DeerGUI/0.1"})
            with urllib.request.urlopen(req, timeout=120) as response:
                total_size = int(response.headers.get("Content-Length", 0))
                chunk_size = 1024 * 1024
                downloaded = 0
                while True:
                    chunk = response.read(chunk_size)
                    if not chunk:
                        break
                    tmp.write(chunk)
                    downloaded += len(chunk)
            
            tmp.close()
            
            print(f"  Extracting ZIP to {texture_dir}...")
            extracted_color_file = None
            with zipfile.ZipFile(tmp_path, "r") as zf:
                for name in zf.namelist():
                    if name.endswith((".png", ".jpg", ".jpeg")):
                        zf.extract(name, texture_dir)
                        if "Color" in name:
                            extracted_color_file = name
            
            tmp_path.unlink(missing_ok=True)
            
            if extracted_color_file:
                # Update configs
                rel_path = f"textures/terrain/{texture_key}/{extracted_color_file}"
                # Find project root (up 3 levels from scripts dir)
                project_root = Path(__file__).parent.parent
                update_scene_configs(project_root, texture_key, rel_path)
                
            print(f"  ✓ Successfully downloaded and extracted {texture_id}.")
            return True
            
    except Exception as e:
        print(f"  ✗ Failed during download or extraction: {e}")
        if 'tmp_path' in locals():
            Path(tmp_path).unlink(missing_ok=True)
        return False

def main():
    parser = argparse.ArgumentParser(description="Download CC0 terrain textures for photo-realistic medieval terrain")
    parser.add_argument("--output", type=str, default=None, help="Output directory (default: apps/deer_gui/assets/textures/terrain)")
    parser.add_argument("--resolution", type=str, default="1K", choices=["1K", "2K", "4K", "8K"], help="Texture resolution (default: 1K)")
    parser.add_argument("--format", type=str, default="PNG", choices=["PNG", "JPG"], help="Texture format preference (default: PNG)")
    parser.add_argument("--texture", type=str, default=None, choices=list(TERRAIN_TEXTURES.keys()), help="Download specific texture only")

    args = parser.parse_args()

    if args.output:
        output_dir = Path(args.output)
    else:
        script_dir = Path(__file__).parent
        output_dir = script_dir.parent / "apps" / "deer_gui" / "assets" / "textures" / "terrain"

    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("  Deer GUI - Terrain Texture Downloader")
    print("  Source: AmbientCG (CC0 PBR textures via API v3)")
    print("=" * 60)
    print(f"Output: {output_dir}")
    print(f"Resolution: {args.resolution}")
    print()

    downloaded = 0
    failed = 0

    for i, (tex_key, info) in enumerate(TERRAIN_TEXTURES.items(), 1):
        if args.texture and args.texture != tex_key:
            continue

        print(f"[{i}/{len(TERRAIN_TEXTURES)}] {info['name']} ({info['ambientcg_id']})")
        
        success = download_ambientcg_texture(tex_key, info["ambientcg_id"], output_dir, args.resolution, args.format)
        
        if success:
            downloaded += 1
        else:
            failed += 1

    print("=" * 60)
    print(f"Results: {downloaded} downloaded, {failed} failed")
    print(f"Assets: {output_dir}")
    print("=" * 60)

    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
