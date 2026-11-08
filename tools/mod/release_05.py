#!/usr/bin/env python3
"""Build and publish the explicitly approved 0.5 with one player asset.

GitHub's automatic Source code ZIP/TAR.GZ point to the verified release tag.
Checksums and reports remain inside the package or Actions, never extra release
attachments. Older releases/tags are neither edited nor replaced.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import zipfile
import package_release
import publish_release

ROOT = Path(__file__).resolve().parents[2]
PROFILE = 'mod/release-0.5.json'
REPO = 'TomRus47/rerac-weapon-evolution'


def descriptor(root=ROOT):
    release = json.loads((root / PROFILE).read_text(encoding='utf-8'))
    development = json.loads((root / 'mod/development.json').read_text(encoding='utf-8'))
    if (release.get('tag') != 'v0.5' or release.get('publish') is not True
            or release.get('download_policy') != 'windows-and-native-source'
            or release.get('asset') != 'ReRAC_Weapon_Evolution_0.5_windows_x64'
            or any(release.get(k) != development.get(k)
                   for k in ('patch', 'config_capability', 'launcher', 'runtime_base', 'public_mod_version'))):
        raise ValueError('Expected the approved 0.5 descriptor and matching development identity.')
    notes = (root / release['notes']).resolve()
    if not notes.is_relative_to(root.resolve()) or not notes.is_file():
        raise ValueError('Release notes must be a tracked file inside this source checkout.')
    return release


def finalize_player(root, assets, commit):
    """Put provenance/instructions in the player ZIP, not in extra downloads."""
    release, _ = publish_release.validate_player(root, assets, commit, PROFILE)
    archive = assets / (release['asset'] + '.zip')
    report_path = assets / 'release-build.json'
    report = json.loads(report_path.read_text(encoding='utf-8'))
    provenance = {k: report[k] for k in ('commit', 'runtime', 'launcher', 'default_ini_sha256', 'icon_sha256')}
    provenance.update(mod_version='0.5', tag='v0.5', runtime_base=release['runtime_base'])
    prefix = release['asset'] + '/'
    replacement = archive.with_suffix('.prepared.zip')
    try:
        with zipfile.ZipFile(archive) as source, zipfile.ZipFile(replacement, 'x', zipfile.ZIP_DEFLATED) as out:
            names = source.namelist()
            if len(names) != len(set(names)):
                raise ValueError('Duplicate player ZIP entries.')
            for name in names:
                path = PurePosixPath(name)
                if not name.startswith(prefix) or path.is_absolute() or '..' in path.parts or '\\' in name:
                    raise ValueError('Unsafe player ZIP path.')
                if name == prefix + 'LEGGIMI.txt':
                    out.writestr(prefix + 'README_FIRST.txt', source.read(name))
                else:
                    out.writestr(name, source.read(name))
            out.writestr(prefix + 'BUILD_INFO.json', json.dumps(provenance, indent=2) + '\n')
            out.writestr(prefix + 'CHANGES_0.5.md', (root / release['notes']).read_bytes())
        with zipfile.ZipFile(replacement) as checked:
            if checked.testzip() is not None:
                raise ValueError('Prepared player ZIP failed integrity verification.')
            for required in ('ReRAC_Weapon_Evolution.exe', 'runtime/rerac.exe', 'weapons.ini', 'README_FIRST.txt'):
                if prefix + required not in checked.namelist():
                    raise ValueError('Incomplete player package: ' + required)
            import sys
            sys.path.insert(0, str(root / 'launcher'))
            from migrate_ini import convert
            canonical = convert((root / 'launcher/weapons.default.ini').read_bytes())[0].removeprefix(b'\xef\xbb\xbf')
            if checked.read(prefix + 'weapons.ini') != canonical:
                raise ValueError('Packaged default INI differs from the approved configuration.')
        os.replace(replacement, archive)
    finally:
        replacement.unlink(missing_ok=True)
    report['sha256'] = publish_release.digest(archive)
    report['download_policy'] = release['download_policy']
    report_path.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    Path(str(archive) + '.sha256').write_text(report['sha256'] + '  ' + archive.name + '\n', encoding='utf-8')
    publish_release.validate_player(root, assets, commit, PROFILE)
    return archive


def api(path, payload=None, optional=False):
    command = ['gh', 'api', 'repos/' + REPO + '/' + path]
    if payload is not None:
        command += ['--method', 'POST', '--input', '-']
    result = subprocess.run(command, input=json.dumps(payload) if payload is not None else None,
                            capture_output=True, text=True, encoding='utf-8')
    if result.returncode:
        if optional and '(HTTP 404)' in result.stderr:
            return None
        raise RuntimeError(result.stderr.strip() or 'GitHub API request failed.')
    return json.loads(result.stdout)


def validate_remote(release, archive, state, commit):
    if state.get('tag_name') != release['tag'] or state.get('target_commitish') != commit:
        raise ValueError('Release identity differs from the checked source commit.')
    assets = state.get('assets', [])
    if len(assets) != 1 or assets[0].get('name') != archive.name:
        raise ValueError('Only the Windows player ZIP may be a release attachment; source is provided by GitHub.')
    if assets[0].get('state') != 'uploaded' or assets[0].get('digest') != 'sha256:' + publish_release.digest(archive):
        raise ValueError('Uploaded Windows ZIP digest differs.')


def upload(assets, root=ROOT):
    if os.environ.get('GH_REPO') != REPO:
        raise ValueError('Publication is scoped to the approved repository.')
    release = descriptor(root)
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
    if not re.fullmatch(r'[0-9a-f]{40}', commit) or os.environ.get('GITHUB_SHA') != commit:
        raise ValueError('Publish only the exact Windows workflow source commit.')
    _, _ = publish_release.validate_player(root, assets, commit, PROFILE)
    archive = assets / (release['asset'] + '.zip')
    # Verify an exact tracked source archive privately. GitHub supplies the
    # downloadable source formats from the same immutable tag; do not duplicate them.
    audit = assets / 'source-audit'
    audit.mkdir(exist_ok=True)
    publish_release.source_archive(root, audit, release['tag'], commit)
    pages = json.loads(subprocess.check_output(['gh', 'api', '--paginate', '--slurp',
        'repos/' + REPO + '/releases?per_page=100'], text=True))
    found = [r for page in pages for r in page if r['tag_name'] == release['tag']]
    if found and (len(found) != 1 or not found[0].get('draft') or found[0].get('target_commitish') != commit):
        raise ValueError('Do not replace a public release or a draft from another commit.')
    ref = api('git/ref/tags/' + release['tag'], optional=True)
    if ref is None:
        api('git/refs', {'ref': 'refs/tags/' + release['tag'], 'sha': commit})
    elif ref.get('object', {}).get('sha') != commit:
        raise ValueError('Existing v0.5 tag points to different source; it is never moved.')
    if not found:
        subprocess.run(['gh', 'release', 'create', release['tag'], str(archive), '--draft',
            '--verify-tag', '--target', commit, '--title', release['title'],
            '--notes-file', str(root / release['notes'])], cwd=root, check=True)
    else:
        subprocess.run(['gh', 'release', 'upload', release['tag'], str(archive), '--clobber'], cwd=root, check=True)
    # The authenticated release list resolves private drafts before publication.
    pages = json.loads(subprocess.check_output(['gh', 'api', '--paginate', '--slurp',
        'repos/' + REPO + '/releases?per_page=100'], text=True))
    draft, = [r for page in pages for r in page if r['tag_name'] == release['tag']]
    state = api('releases/' + str(draft['id']))
    validate_remote(release, archive, state, commit)
    if not state.get('draft') or api('git/ref/tags/' + release['tag'])['object']['sha'] != commit:
        raise ValueError('Release/tag changed before publication.')
    subprocess.run(['gh', 'release', 'edit', release['tag'], '--draft=false', '--prerelease=false', '--latest'], cwd=root, check=True)
    state = api('releases/' + str(draft['id']))
    validate_remote(release, archive, state, commit)
    if state.get('draft') or state.get('prerelease'):
        raise ValueError('The approved 0.5 release was not published.')
    print('Published: ' + state['html_url'], flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=('package', 'publish'))
    parser.add_argument('--assets', type=Path, default=ROOT / 'release-assets')
    args = parser.parse_args()
    descriptor()
    if args.stage == 'package':
        package_release.main(PROFILE)
        commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
        finalize_player(ROOT, ROOT / 'launcher/dist', commit)
    else:
        upload(args.assets.resolve())


if __name__ == '__main__':
    main()
