#!/usr/bin/env python3
"""Engine-only native/WASM verification. --browser adds loopback-only Grim rendering smoke.
Requires Cargo, Trunk, wasm32 target; browser option additionally requires Playwright
and Edge or Playwright Chromium. No deployment or external game repository required.
"""
import argparse
import functools
import hashlib
import http.server
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / '.slipstream' / 'engine-verification'
GAME = ROOT / 'examples' / 'grim_delivery'


def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--browser', action='store_true')
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    report = {'passed': False, 'commands': [], 'artifacts': [], 'browser_performed': False}
    start = time.monotonic()

    def run(command, cwd=ROOT):
        print('+', ' '.join(command), flush=True)
        result = subprocess.run(command, cwd=cwd, capture_output=True, text=True,
            encoding='utf-8', errors='replace', timeout=600)
        name = f'{len(report["commands"])+1:02d}.log'
        (OUT / name).write_text(result.stdout + result.stderr, encoding='utf-8')
        report['commands'].append({'command': command, 'exit': result.returncode, 'log': name})
        if result.returncode:
            raise RuntimeError(f'Command failed; see {OUT / name}')

    try:
        for command in [
            ['cargo', 'test', '--workspace', '--locked'],
            ['cargo', 'build', '--workspace', '--locked'],
            ['cargo', 'clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings'],
            ['cargo', 'fmt', '--all', '--', '--check'],
            ['cargo', 'check', '-p', 'grim_delivery', '--target', 'wasm32-unknown-unknown', '--locked'],
        ]:
            run(command)
        run(['trunk', 'build', '--release', '--locked'], GAME)
        dist = GAME / 'dist'
        modules = list(dist.glob('*.wasm'))
        if len(modules) != 1 or modules[0].read_bytes()[:8] != b'\x00asm\x01\x00\x00\x00':
            raise RuntimeError('Expected one valid WASM module')
        for path in sorted(dist.iterdir()):
            if path.is_file():
                data = path.read_bytes()
                report['artifacts'].append({'file': path.name, 'bytes': len(data),
                    'sha256': hashlib.sha256(data).hexdigest()})
        if args.browser:
            from playwright.sync_api import sync_playwright
            server = http.server.ThreadingHTTPServer(('127.0.0.1', 0),
                functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(dist)))
            worker = threading.Thread(target=server.serve_forever, daemon=True)
            worker.start()
            try:
                with sync_playwright() as pw:
                    edge = Path(os.environ.get('PROGRAMFILES(X86)', 'C:/Program Files (x86)')) / 'Microsoft/Edge/Application/msedge.exe'
                    options = {'executable_path': str(edge)} if edge.exists() else {}
                    browser = pw.chromium.launch(headless=True, args=['--enable-unsafe-swiftshader',
                        '--use-angle=swiftshader', '--disable-gpu-sandbox'], **options)
                    try:
                        page = browser.new_page(viewport={'width': 1280, 'height': 720})
                        errors, console = [], []
                        page.on('pageerror', lambda error: errors.append(str(error)))
                        page.on('console', lambda message: console.append(message.text))
                        page.goto(f'http://127.0.0.1:{server.server_port}/', wait_until='networkidle')
                        page.wait_for_function("document.querySelector('canvas')?.width > 1")
                        page.wait_for_timeout(1500)
                        page.keyboard.press('Space')
                        page.set_viewport_size({'width': 960, 'height': 640})
                        page.wait_for_timeout(300)
                        page.screenshot(path=str(OUT / 'browser.png'))
                        if errors or not any('first frame presented:' in line for line in console):
                            raise RuntimeError(f'No successful presentation or page errors: {errors}')
                        report['browser'] = {'page_errors': errors, 'first_frame_presented': True,
                            'keyboard_and_resize_sent': True, 'human_playtest': False}
                        report['browser_performed'] = True
                    finally:
                        browser.close()
            finally:
                server.shutdown()
                server.server_close()
                worker.join(timeout=5)
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        print('FAIL:', error)
    report['seconds'] = round(time.monotonic() - start, 2)
    (OUT / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print('PASS' if report['passed'] else 'FAIL', OUT / 'report.json')
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
