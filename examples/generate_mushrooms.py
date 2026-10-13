#!/usr/bin/env python3
"""Generate N original four-frame sprites through ONE CLI session; no dependencies."""
import argparse
import json
import subprocess
from pathlib import Path

PALETTE = [[0, 0, 0, 0], [35, 36, 49, 255], [190, 59, 63, 255], [239, 105, 78, 255],
           [248, 216, 162, 255], [255, 245, 213, 255], [154, 104, 78, 255], [58, 66, 69, 100]]

def mushroom(offset):
    pixels = [PALETTE[0][:] for _ in range(32 * 32)]
    def put(x, y, color):
        if 0 <= x < 32 and 0 <= y + offset < 32:
            pixels[(y + offset) * 32 + x] = PALETTE[color]
    for y in range(15, 27):
        for x in range(11, 22):
            edge = x in (11, 21) or y == 26
            put(x, y, 1 if edge else (6 if x > 18 else 4))
    for y in range(7, 19):
        for x in range(4, 29):
            dx, dy = (x - 16) / 12, (y - 16) / 10
            if dx * dx + dy * dy <= 1 and y <= 18:
                edge = abs(dx) > .85 or y == 18 or ((x - 16) / 12) ** 2 + ((y - 15) / 10) ** 2 > 1
                put(x, y, 1 if edge else (3 if x < 16 and y < 13 else 2))
    for cx, cy in [(10, 13), (19, 10), (23, 15)]:
        for dy in range(2):
            for dx in range(3):
                put(cx + dx, cy + dy, 5)
    for x in (14, 18):
        put(x, 21, 1); put(x, 22, 1)
    put(16, 24, 6)
    return pixels

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--engine', required=True)
    p.add_argument('--workspace', required=True)
    p.add_argument('--count', type=int, default=1)
    a = p.parse_args()
    if not 1 <= a.count <= 1000:
        p.error('--count must be 1..1000')
    root = Path(a.workspace).resolve()
    root.mkdir(parents=True, exist_ok=True)
    process = subprocess.Popen([str(Path(a.engine).resolve()), '--workspace', str(root)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    sequence = 0
    def command(document, name, args=None):
        nonlocal sequence
        sequence += 1
        process.stdin.write(json.dumps({'request_id': f'python-{sequence}', 'document_id': document, 'command': name, 'args': args or {}}) + '\n')
        process.stdin.flush()
        response = json.loads(process.stdout.readline())
        if not response['ok']:
            raise RuntimeError(response['error'])
        return response['result']
    try:
        for n in range(a.count):
            document = f'mushroom-{n:03d}'
            command(document, 'document.create', {'width': 32, 'height': 32})
            command(document, 'layer.update', {'layer_id': 1, 'name': 'Ground shadow'})
            layer = command(document, 'layer.create', {'name': 'Mushroom'})['results'][0]['layer_id']
            shadow = [PALETTE[0][:] for _ in range(1024)]
            for y in range(27, 30):
                for x in range(9, 24):
                    if ((x - 16) / 7) ** 2 + ((y - 28) / 1.5) ** 2 <= 1:
                        shadow[y * 32 + x] = PALETTE[7]
            command(document, 'command.batch', {'commands': [
                {'command': 'palette.set', 'args': {'colors': PALETTE}},
                {'command': 'canvas.patch', 'args': {'layer_id': 1, 'frame_id': 2, 'x': 0, 'y': 0, 'width': 32, 'height': 32, 'pixels': shadow}},
            ]})
            frame_ids = [2]
            for i, offset in enumerate([0, -1, 0, 1]):
                frame = 2 if i == 0 else command(document, 'frame.add', {'source_frame_id': 2, 'duration_ms': 160})['results'][0]['frame_id']
                if i:
                    frame_ids.append(frame)
                command(document, 'canvas.patch', {'layer_id': layer, 'frame_id': frame, 'x': 0, 'y': 0, 'width': 32, 'height': 32, 'pixels': mushroom(offset)})
            command(document, 'animation.tag', {'name': 'idle', 'from_frame': frame_ids[0], 'to_frame': frame_ids[-1]})
            command(document, 'document.save', {'path': document + '.spritelab', 'overwrite': True})
            command(document, 'export.png', {'path': document + '.png', 'frame_id': 2, 'overwrite': True})
            atlas = command(document, 'export.spritesheet', {'path': document + '-idle.png', 'columns': 4, 'padding': 1, 'overwrite': True})
            (root / (document + '-idle.json')).write_text(json.dumps(atlas['metadata'], indent=2) + '\n')
            command(document, 'document.close')
        print(f'Generated {a.count} sprite(s), each with four frames, in {root}')
    finally:
        process.stdin.close()
        process.wait(timeout=10)

if __name__ == '__main__':
    main()
