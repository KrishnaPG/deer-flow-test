#!/usr/bin/env python3
"""Download photogrammetry rocks from OpenGameArt for the medieval scene."""

import os
import sys
import urllib.request
import zipfile
import shutil

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS_DIR = os.path.join(BASE_DIR, "apps", "deer_gui", "assets", "models", "rocks")

ROCKS = [
    {
        "name": "photogrammetric_rock_1",
        "url": "https://opengameart.org/sites/default/files/Free%20Photogrammetric%20Rock.7z",
        "archive": "Free Photogrammetric Rock.7z",
    },
    {
        "name": "photogrammetric_rock_2",
        "url": "https://opengameart.org/sites/default/files/Free%20Photogrammetric%20Rock%202.7z",
        "archive": "Free Photogrammetric Rock 2.7z",
    },
    {
        "name": "photogrammetric_rock_cluster",
        "url": "https://opengameart.org/sites/default/files/Free%20Photogrammetric%20Rock%20-%20Cluster.7z",
        "archive": "Free Photogrammetric Rock - Cluster.7z",
    },
]


def download_file(url, dest):
    """Download a file with progress."""
    print(f"Downloading {os.path.basename(dest)}...")
    try:
        urllib.request.urlretrieve(url, dest)
        print(f"  -> Saved to {dest}")
        return True
    except Exception as e:
        print(f"  -> FAILED: {e}")
        return False


def main():
    os.makedirs(ASSETS_DIR, exist_ok=True)

    for rock in ROCKS:
        archive_path = os.path.join(ASSETS_DIR, rock["archive"])

        if os.path.exists(archive_path):
            print(f"Already downloaded: {rock['name']}")
            continue

        if not download_file(rock["url"], archive_path):
            print(f"ERROR: Could not download {rock['name']}")
            print(f"URL: {rock['url']}")
            print("You may need to download manually from OpenGameArt and place in:")
            print(f"  {ASSETS_DIR}")
            continue

    print("\nDownload complete. Archives are in:")
    print(f"  {ASSETS_DIR}")
    print("\nNote: These are .7z archives. You will need to extract them manually")
    print("or install p7zip and run:")
    print(f"  cd {ASSETS_DIR} && 7z x '*.7z'")
    print(
        "\nAfter extraction, convert OBJ/FBX files to GLB format for Bevy compatibility."
    )


if __name__ == "__main__":
    main()
