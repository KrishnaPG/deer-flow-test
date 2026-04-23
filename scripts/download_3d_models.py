#!/usr/bin/env python3
"""
Download CC0 3D models and textures for medieval landscape scene.

Sources (all CC0 / free):
  - Poly Haven: https://polyhaven.com - CC0 HDRIs (direct downloads)
  - AmbientCG: https://ambientcg.com - CC0 PBR materials (direct downloads)
  - Kenney: https://kenney.nl - CC0 game assets (manual download required)

Usage:
    python scripts/download_3d_models.py [--output DIR] [--category CATEGORY]

Categories:
    sky         - HDRI skyboxes from Poly Haven
    textures    - PBR terrain textures from AmbientCG
    all         - All categories

Note: Kenney 3D models require manual download from https://kenney.nl
      as they don't provide stable direct download URLs.

Defaults:
    --output: apps/deer_gui/assets
"""

import argparse
import os
import sys
import urllib.request
import urllib.error
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
# AmbientCG PBR textures (CC0) - direct download URLs
# ---------------------------------------------------------------------------

AMBIENTCG_TEXTURES = {
    "grass": {
        "name": "Grass 01",
        "id": "Grass01",
        "url_1k": "https://dl.polyhaven.org/file/ph-assets/AmbientCG/textures/Grass01_1K.zip",
        "description": "Green grass with normal and roughness maps",
    },
    "dirt": {
        "name": "Dirt 01",
        "id": "Dirt01",
        "url_1k": "https://dl.polyhaven.org/file/ph-assets/AmbientCG/textures/Dirt01_1K.zip",
        "description": "Brown dirt/soil texture",
    },
    "rock": {
        "name": "Rock 01",
        "id": "Rock01",
        "url_1k": "https://dl.polyhaven.org/file/ph-assets/AmbientCG/textures/Rock01_1K.zip",
        "description": "Grey rock texture",
    },
    "snow": {
        "name": "Snow 01",
        "id": "Snow01",
        "url_1k": "https://dl.polyhaven.org/file/ph-assets/AmbientCG/textures/Snow01_1K.zip",
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


def download_and_extract_zip(url: str, extract_dir: Path, timeout: int = 300) -> bool:
    """Download a ZIP file and extract its contents."""
    import tempfile
    import zipfile

    with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as tmp:
        tmp_path = Path(tmp.name)

        print(f"  Downloading ZIP: {url}")
        try:
            req = urllib.request.Request(
                url, headers={"User-Agent": "DeerGUI/0.1 (Bevy Game Engine)"}
            )

            with urllib.request.urlopen(req, timeout=timeout) as response:
                data = response.read()

            tmp.write(data)
            tmp.close()

            size_mb = len(data) / (1024 * 1024)
            print(f"  ✓ Downloaded ZIP ({size_mb:.1f} MB)")

        except Exception as e:
            print(f"  ✗ Download failed: {e}")
            tmp_path.unlink(missing_ok=True)
            return False

        # Extract ZIP
        print(f"  Extracting to: {extract_dir}")
        try:
            with zipfile.ZipFile(tmp_path, "r") as zf:
                zf.extractall(extract_dir)
            print(f"  ✓ Extracted {len(zf.namelist())} files")
        except Exception as e:
            print(f"  ✗ Extraction failed: {e}")
            tmp_path.unlink(missing_ok=True)
            return False

        tmp_path.unlink(missing_ok=True)
        return True


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


def download_ambientcg_texture(texture_key: str, output_dir: Path) -> bool:
    """Download AmbientCG PBR texture and extract to proper directory."""
    import tempfile
    import zipfile

    tex_info = AMBIENTCG_TEXTURES.get(texture_key)
    if not tex_info:
        print(f"  ✗ Unknown texture: {texture_key}")
        return False

    tex_dir = output_dir / "textures" / "terrain" / texture_key
    tex_dir.mkdir(parents=True, exist_ok=True)

    # Download ZIP
    with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as tmp:
        tmp_path = Path(tmp.name)

        print(f"  Downloading: {tex_info['url_1k']}")
        try:
            req = urllib.request.Request(
                tex_info["url_1k"],
                headers={"User-Agent": "DeerGUI/0.1 (Bevy Game Engine)"},
            )

            with urllib.request.urlopen(req, timeout=300) as response:
                data = response.read()

            tmp.write(data)
            tmp.close()

            size_mb = len(data) / (1024 * 1024)
            print(f"  ✓ Downloaded ({size_mb:.1f} MB)")

        except Exception as e:
            print(f"  ✗ Download failed: {e}")
            tmp_path.unlink(missing_ok=True)
            return False

        # Extract ZIP and organize files
        try:
            with zipfile.ZipFile(tmp_path, "r") as zf:
                for name in zf.namelist():
                    if name.endswith((".png", ".jpg", ".jpeg", ".tga")):
                        # Extract to texture directory
                        zf.extract(name, tex_dir)
                        print(f"  ✓ {name}")
        except Exception as e:
            print(f"  ✗ Extraction failed: {e}")
            tmp_path.unlink(missing_ok=True)
            return False

        tmp_path.unlink(missing_ok=True)
        return True


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
    print("  Sources: PolyHaven, AmbientCG")
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

        for tex_key in textures_to_download:
            tex_info = AMBIENTCG_TEXTURES[tex_key]
            print(f"\n  [{tex_key}] {tex_info['name']}")
            print(f"    {tex_info['description']}")
            download_ambientcg_texture(tex_key, output_dir)

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
