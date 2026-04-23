#!/usr/bin/env python3
"""
Download CC0 3D models and textures for medieval landscape scene.

Sources (all CC0 / free):
  - Poly Haven: https://polyhaven.com - CC0 HDRIs (direct downloads)
  - AmbientCG: https://ambientcg.com - CC0 PBR materials (via v3 API)
  - Kenney: https://kenney.nl - CC0 game assets (manual download required)

Usage:
    python scripts/download_3d_models.py [--output DIR] [--category CATEGORY]

Categories:
    sky         - HDRI skyboxes from Poly Haven
    textures    - PBR terrain textures from AmbientCG
    all         - All categories

Note: Kenney 3D models require manual download from https://kenney.nl
      as they don't provide stable direct download URLs.
"""

import argparse
import os
import sys
import urllib.request
import urllib.error
import tempfile
import zipfile
import json
from pathlib import Path


# ---------------------------------------------------------------------------
# Poly Haven HDRI skyboxes (CC0) - direct download URLs
# ---------------------------------------------------------------------------

POLYHAVEN_HDRIS = {
    "kloofendal_48d_partly_cloudy": {
        "name": "Kloofendal - Partly Cloudy",
        "url_1k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/kloofendal_48d_partly_cloudy_puresky_1k.hdr",
        "url_2k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/2k/kloofendal_48d_partly_cloudy_puresky_2k.hdr",
        "url_4k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/4k/kloofendal_48d_partly_cloudy_puresky_4k.hdr",
        "description": "Partly cloudy sky over green valley, golden hour lighting",
    },
    "table_mountain_2": {
        "name": "Table Mountain 2",
        "url_1k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/table_mountain_2_1k.hdr",
        "url_2k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/2k/table_mountain_2_2k.hdr",
        "url_4k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/4k/table_mountain_2_4k.hdr",
        "description": "Mountain landscape with dramatic clouds",
    },
    "meadow": {
        "name": "Meadow",
        "url_1k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/meadow_1k.hdr",
        "url_2k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/2k/meadow_2k.hdr",
        "url_4k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/4k/meadow_4k.hdr",
        "description": "Open meadow with soft lighting, ideal for medieval scenes",
    },
    "brown_photostudio_02": {
        "name": "Brown Photo Studio 02",
        "url_1k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/brown_photostudio_02_1k.hdr",
        "url_2k": "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/2k/brown_photostudio_02_2k.hdr",
        "description": "Neutral studio lighting, good for testing",
    },
}

# ---------------------------------------------------------------------------
# AmbientCG PBR textures (CC0)
# ---------------------------------------------------------------------------

AMBIENTCG_TEXTURES = {
    "grass": {
        "name": "Grass 01",
        "id": "Grass01",
        "description": "Green grass with normal and roughness maps",
    },
    "dirt": {
        "name": "Dirt 01",
        "id": "Dirt01",
        "description": "Brown dirt/soil texture",
    },
    "rock": {
        "name": "Rock 01",
        "id": "Rock01",
        "description": "Grey rock texture",
    },
    "snow": {
        "name": "Snow 01",
        "id": "Snow01",
        "description": "White snow texture",
    },
}


# ---------------------------------------------------------------------------
# Download utilities
# ---------------------------------------------------------------------------

def download_file(url: str, dest: Path, timeout: int = 300) -> bool:
    """Download a file with progress reporting."""
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


def download_polyhaven_hdri(
    hdri_key: str, output_dir: Path, resolution: str = "2k"
) -> bool:
    """Download a Poly Haven HDRI skybox."""
    hdri_info = POLYHAVEN_HDRIS.get(hdri_key)
    if not hdri_info:
        print(f"  ✗ Unknown HDRI: {hdri_key}")
        return False

    sky_dir = output_dir / "sky"
    sky_dir.mkdir(parents=True, exist_ok=True)

    url_key = f"url_{resolution}"
    url = hdri_info.get(url_key, hdri_info.get("url_1k"))

    dest = sky_dir / f"{hdri_key}.hdr"
    return download_file(url, dest)


def download_ambientcg_texture(texture_key: str, output_dir: Path, resolution: str = "1K", format_pref: str = "PNG") -> bool:
    """Download AmbientCG PBR texture using v3 API and extract to proper directory."""
    tex_info = AMBIENTCG_TEXTURES.get(texture_key)
    if not tex_info:
        print(f"  ✗ Unknown texture: {texture_key}")
        return False

    texture_id = tex_info["id"]
    tex_dir = output_dir / "textures" / "terrain" / texture_key
    tex_dir.mkdir(parents=True, exist_ok=True)

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
    
    target_attr = f"{resolution}-{format_pref}"
    download_url = None
    
    for dl in downloads:
        if dl.get("attributes") == target_attr and dl.get("extension") == "zip":
            download_url = dl.get("url")
            break
            
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
            
            print(f"  Extracting ZIP to {tex_dir}...")
            with zipfile.ZipFile(tmp_path, "r") as zf:
                for name in zf.namelist():
                    if name.endswith((".png", ".jpg", ".jpeg")):
                        zf.extract(name, tex_dir)
                        print(f"  ✓ {name}")
            
            tmp_path.unlink(missing_ok=True)
            return True
            
    except Exception as e:
        print(f"  ✗ Failed during download or extraction: {e}")
        if 'tmp_path' in locals():
            Path(tmp_path).unlink(missing_ok=True)
        return False


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    parser = argparse.ArgumentParser(
        description="Download CC0 assets for medieval landscape scene"
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Output directory (default: apps/deer_gui/assets)",
    )
    parser.add_argument(
        "--category",
        type=str,
        default="all",
        choices=["sky", "textures", "all"],
        help="Asset category to download",
    )
    parser.add_argument(
        "--hdri",
        type=str,
        default=None,
        choices=list(POLYHAVEN_HDRIS.keys()) + ["all"],
        help="HDRI skybox to download from Poly Haven",
    )
    parser.add_argument(
        "--resolution",
        type=str,
        default="2k",
        choices=["1k", "2k", "4k"],
        help="HDRI resolution (default: 2k)",
    )

    args = parser.parse_args()

    # Resolve output directory
    if args.output:
        output_dir = Path(args.output)
    else:
        script_dir = Path(__file__).parent
        project_root = script_dir.parent
        output_dir = project_root / "apps" / "deer_gui" / "assets"

    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("  Deer GUI - CC0 Asset Downloader")
    print("  Sources: PolyHaven, AmbientCG (API v3)")
    print("=" * 60)
    print(f"Output: {output_dir}")
    print(f"Category: {args.category}")
    print()

    # Download HDRI skyboxes
    if args.category in ("all", "sky"):
        print(f"\n{'=' * 40}")
        print("[sky] Poly Haven HDRI Skyboxes")
        print(f"{'=' * 40}")

        hdris_to_download = []
        if args.hdri == "all":
            hdris_to_download = list(POLYHAVEN_HDRIS.keys())
        elif args.hdri:
            hdris_to_download = [args.hdri]
        elif args.category == "all":
            hdris_to_download = ["meadow", "kloofendal_48d_partly_cloudy"]

        for hdri_key in hdris_to_download:
            hdri_info = POLYHAVEN_HDRIS[hdri_key]
            print(f"\n  [{hdri_key}] {hdri_info['name']}")
            print(f"    {hdri_info['description']}")
            download_polyhaven_hdri(hdri_key, output_dir, args.resolution)

    # Download PBR textures
    if args.category in ("all", "textures"):
        print(f"\n{'=' * 40}")
        print("[textures] AmbientCG PBR Textures")
        print(f"{'=' * 40}")

        textures_to_download = list(AMBIENTCG_TEXTURES.keys())

        # For textures we typically want 1K by default, but we can respect args.resolution if it fits (1k -> 1K)
        tex_res = args.resolution.upper()

        for tex_key in textures_to_download:
            tex_info = AMBIENTCG_TEXTURES[tex_key]
            print(f"\n  [{tex_key}] {tex_info['name']}")
            print(f"    {tex_info['description']}")
            download_ambientcg_texture(tex_key, output_dir, resolution=tex_res)

    print(f"\n{'=' * 60}")
    print("Done! Assets downloaded to:", output_dir)
    print()
    print("For Kenney 3D models (buildings, characters, foliage):")
    print("  1. Visit https://kenney.nl/assets")
    print("  2. Download: Nature Pack, Medieval Village, Prototype Characters")
    print("  3. Extract glTF files to apps/deer_gui/assets/models/")
    print("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())
