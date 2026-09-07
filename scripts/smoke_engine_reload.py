#!/usr/bin/env python3
"""Native last-known-good reload regression using disposable copies of authored assets.
Build first: cargo build -p sme_game --locked. Opens one native window briefly.
Checks logged reload boundaries/process survival, not pixel identity or human usability.
Never modifies repository assets. No third-party Python dependencies.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / '.slipstream' / 'foundation-reload'


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    report = {'passed': False, 'checks': [], 'pixel_comparison': False}
    binary = ROOT / 'target' / 'debug' / ('sme_game.exe' if os.name == 'nt' else 'sme_game')
    log_path = OUT / 'native.log'
    proc = None
    try:
        with tempfile.TemporaryDirectory(prefix='sme-reload-') as tmp:
            work = Path(tmp)
            for name in ['scenes', 'generated', 'collision', 'scripts', 'textures']:
                shutil.copytree(ROOT / 'assets' / name, work / 'assets' / name)
            with log_path.open('w', encoding='utf-8') as log:
                proc = subprocess.Popen([str(binary)], cwd=work, stdout=log,
                    stderr=subprocess.STDOUT, env={**os.environ, 'RUST_LOG': 'info'})
                def text():
                    return log_path.read_text(encoding='utf-8', errors='replace')

                def wait_for(needle, offset=0):
                    deadline = time.monotonic() + 15
                    while time.monotonic() < deadline:
                        if proc.poll() is not None:
                            raise RuntimeError(f'Native app exited: {proc.returncode}')
                        if needle in text()[offset:]:
                            return
                        time.sleep(0.05)
                    raise RuntimeError(f'Timed out waiting for {needle!r}; see {log_path}')

                def update(path, data):
                    stamp = max(time.time(), path.stat().st_mtime + 1)
                    path.write_bytes(data)
                    os.utime(path, (stamp, stamp))

                try:
                    wait_for('Lua script loaded:')
                    time.sleep(0.3)
                    atlas_path = work / 'assets/generated/m4_sample_atlas.json'
                    scene_path = work / 'assets/scenes/m4_scene.json'
                    atlas_bytes = atlas_path.read_bytes()
                    scene_bytes = scene_path.read_bytes()
                    atlas = json.loads(atlas_bytes)
                    scene = json.loads(scene_bytes)
                    referenced = next(s['sprite_id'] for layer in scene['layers']
                        for s in layer['sprites'] if 'sprite_id' in s)
                    atlas['sprites'] = [s for s in atlas['sprites'] if s['sprite_id'] != referenced]
                    offset = len(text())
                    update(atlas_path, json.dumps(atlas).encode())
                    wait_for('Atlas reload failed', offset)
                    time.sleep(0.2)
                    assert 'Skipping sprite' not in text()[offset:]
                    report['checks'].append('missing sprite replacement rejected; live rendering retains references')

                    texture_path = work / json.loads(atlas_bytes)['texture']['path']
                    texture_bytes = texture_path.read_bytes()
                    texture_path.write_bytes(b'corrupt PNG')
                    offset = len(text())
                    update(atlas_path, atlas_bytes)
                    wait_for('Atlas reload failed', offset)
                    assert 'Texture ' in text()[offset:]
                    report['checks'].append('corrupt same-path PNG rejected without panic')

                    texture_path.write_bytes(texture_bytes)
                    offset = len(text())
                    update(atlas_path, atlas_bytes)
                    wait_for('Atlas reloaded', offset)
                    report['checks'].append('valid same-path atlas pixels staged and replacement committed')

                    scene['animations'] = ['assets/missing-animation.json']
                    offset = len(text())
                    update(scene_path, json.dumps(scene).encode())
                    wait_for('anim load error', offset)
                    assert 'Scene reloaded' not in text()[offset:]
                    report['checks'].append('missing scene dependency aborts scene commit')
                    offset = len(text())
                    update(scene_path, scene_bytes)
                    wait_for('Scene reloaded', offset)
                    report['checks'].append('valid scene reload recovers after rejection')
                    assert 'panicked' not in text()
                    report['passed'] = True
                finally:
                    if proc.poll() is None:
                        proc.terminate()
                    proc.wait(timeout=10)
    except Exception as error:
        report['error'] = str(error)
    (OUT / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(('PASS' if report['passed'] else 'FAIL'), OUT / 'report.json')
    if not report['passed']:
        print(report.get('error'))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
