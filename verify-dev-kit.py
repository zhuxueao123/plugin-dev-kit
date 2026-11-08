#!/usr/bin/env python3
"""Verify published tool deliveries and independently versioned project content."""
import argparse
import hashlib
import json
from pathlib import Path


def verify(root):
    if (root / 'plugin-workspace').exists():
        raise ValueError('Legacy embedded plugin-workspace exists; move projects outside the tool checkout')
    manifest = json.loads((root / 'content/manifest.json').read_text())
    for relative, expected in manifest['files'].items():
        if not relative.startswith('docs/dev-kit/') or '..' in Path(relative).parts:
            raise ValueError(f'Unsafe managed content: {relative}')
        path = root / 'content' / relative
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f'Content checksum mismatch: {relative}')
    record = json.loads((root / 'CLI_VERSION.json').read_text())
    for platform, item in record['platforms'].items():
        path = root / item['path']
        if hashlib.sha256(path.read_bytes()).hexdigest() != item['sha256']:
            raise ValueError(f'Executable checksum mismatch: {platform}')
    print(f"Verified CLI {record['version']}, content {manifest['version']}, {len(manifest['files'])} managed files")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=['local', 'delivery'], default='local')
    parser.parse_args()
    verify(Path(__file__).resolve().parent)
