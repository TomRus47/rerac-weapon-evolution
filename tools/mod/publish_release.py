#!/usr/bin/env python3
"""Publish checked Windows/source assets, or replace the explicitly approved v0.3."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def api(repo: str, path: str) -> dict:
    return json.loads(subprocess.check_output(['gh', 'api', f'repos/{repo}/{path}'], text=True))


def validate_player(root: Path, assets: Path, commit: str,
                    profile: str = 'mod/release.json') -> tuple[dict, list[Path]]:
    release = json.loads((root / profile).read_text())
    report_path = assets / 'release-build.json'
    report = json.loads(report_path.read_text())
    archive = assets / (release['asset'] + '.zip')
    checksum = Path(str(archive) + '.sha256')
    runtime = report.get('runtime', {})
    if (not re.fullmatch(r'[0-9a-f]{40}', commit) or report.get('commit') != commit
            or report.get('release') != release['tag'] or report.get('launcher') != release['launcher']
            or report.get('archive') != archive.name or report.get('sha256') != digest(archive)
            or runtime.get('weapon_evolution_patch') != release['patch']
            or runtime.get('weapon_evolution_config') != release['config_capability']
            or runtime.get('weapon_evolution_audio_diagnostics') is not False
            or str(runtime.get('version', '')).endswith('-fixed')
            or report.get('default_ini_sha256') != digest(root / 'launcher/weapons.default.ini')
            or report.get('icon_sha256') != digest(root / 'launcher/launcher.ico')):
        raise ValueError('Release assets do not match the checked source/runtime identity.')
    if checksum.read_text().split() != [digest(archive), archive.name]:
        raise ValueError('Player ZIP checksum sidecar does not match.')
    with zipfile.ZipFile(archive) as package:
        if package.testzip() is not None:
            raise ValueError('Player ZIP is damaged.')
    return release, [archive, checksum, report_path]


def source_archive(root: Path, assets: Path, tag: str, commit: str) -> list[Path]:
    if not re.fullmatch(r'v\d+\.\d+(?:\.\d+)?(?:-[a-zA-Z0-9][a-zA-Z0-9.-]*)?', tag):
        raise ValueError('Invalid mod release tag.')
    stem = f'ReRAC_Weapon_Evolution_{tag[1:]}_source'
    archive = assets / (stem + '.zip')
    subprocess.run(['git', 'archive', '--format=zip', f'--prefix={stem}/', '-o', str(archive), commit], cwd=root, check=True)
    listing = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', '-z', commit], cwd=root)
    expected = {stem + '/' + name.decode() for name in listing.split(b'\0') if name}
    with zipfile.ZipFile(archive) as package:
        actual = {entry.filename for entry in package.infolist() if not entry.is_dir()}
        if actual != expected or package.testzip() is not None:
            raise ValueError('Source ZIP does not match the exact tracked commit.')
    checksum = Path(str(archive) + '.sha256')
    checksum.write_text(f'{digest(archive)}  {archive.name}\n', encoding='utf-8')
    return [archive, checksum]


def verify_uploaded(repo: str, tag: str, files: list[Path]) -> None:
    remote = {entry['name']: entry for entry in api(repo, f'releases/tags/{tag}')['assets']}
    for path in files:
        item = remote.get(path.name, {})
        if item.get('state') != 'uploaded' or item.get('digest') != 'sha256:' + digest(path):
            raise ValueError('Uploaded release asset digest differs: ' + path.name)


def replace_release(repo: str, release: dict, files: list[Path], commit: str, notes: Path) -> None:
    """Move the tag only after all replacement files have uploaded and verified."""
    if release['tag'] != 'v0.3' or not re.fullmatch(r'[0-9a-f]{40}', release.get('replace_tag_sha', '')):
        raise ValueError('Replacement is authorized only for the recorded v0.3 tag.')
    tag = release['tag']
    expected = release['replace_tag_sha']
    if api(repo, f'git/ref/tags/{tag}')['object']['sha'] != expected:
        raise ValueError('The release tag changed since this replacement was prepared.')
    subprocess.run(['gh', 'release', 'upload', tag, *map(str, files), '--clobber'], check=True)
    verify_uploaded(repo, tag, files)
    # Recheck the lease after uploads, before replacing GitHub's source-code tag.
    if api(repo, f'git/ref/tags/{tag}')['object']['sha'] != expected:
        raise ValueError('The release tag changed while assets were uploading.')
    subprocess.run(['gh', 'api', '--method', 'PATCH', f'repos/{repo}/git/refs/tags/{tag}', '--input', '-'],
                   input=json.dumps({'sha': commit, 'force': True}), text=True, check=True)
    if api(repo, f'git/ref/tags/{tag}')['object']['sha'] != commit:
        raise ValueError('The release tag did not move to the packaged source commit.')
    subprocess.run(['gh', 'release', 'edit', tag, '--title', release['title'], '--notes-file', str(notes),
                    '--target', commit, '--draft=false', '--latest'], check=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--assets', type=Path, required=True)
    parser.add_argument('--publish', action='store_true')
    parser.add_argument('--replace-existing', action='store_true')
    args = parser.parse_args()
    if args.replace_existing and not args.publish:
        raise ValueError('Replacing an existing release requires explicit publication authorization.')
    repo = os.environ['GH_REPO']
    commit = os.environ['GITHUB_SHA']
    release, files = validate_player(ROOT, args.assets.resolve(), commit)
    if args.publish and release['tag'] != 'v0.3':
        raise ValueError('Publication authorization is restricted to v0.3.')
    files += source_archive(ROOT, args.assets.resolve(), release['tag'], commit)
    notes = ROOT / release['notes']
    existing = subprocess.run(['gh', 'release', 'view', release['tag']], capture_output=True)
    if existing.returncode == 0:
        if args.replace_existing:
            replace_release(repo, release, files, commit, notes)
        else:
            print('Existing release preserved. Replacement requires its explicit publication action.')
        return
    if args.replace_existing:
        raise ValueError('The release to replace no longer exists.')
    subprocess.run(['gh', 'release', 'create', release['tag'], *map(str, files), '--draft', '--target', commit,
                    '--title', release['title'], '--notes-file', str(notes)], check=True)
    verify_uploaded(repo, release['tag'], files)
    if args.publish:
        subprocess.run(['gh', 'release', 'edit', release['tag'], '--draft=false', '--latest'], check=True)
    print(f'Published matching player/source assets from {commit}.')


if __name__ == '__main__':
    main()
