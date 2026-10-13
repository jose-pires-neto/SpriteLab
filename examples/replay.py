#!/usr/bin/env python3
"""Replay the same JSONL command stream as any other adapter. No editing rules."""
import argparse
import subprocess
from pathlib import Path
p = argparse.ArgumentParser()
p.add_argument('--engine', required=True)
p.add_argument('--workspace', required=True)
p.add_argument('--transcript', required=True)
a = p.parse_args()
result = subprocess.run([a.engine, '--workspace', a.workspace], input=Path(a.transcript).read_bytes(), capture_output=True, check=True)
print(result.stdout.decode(), end='')
