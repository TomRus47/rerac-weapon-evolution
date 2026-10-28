#!/usr/bin/env python3
"""Build exact player/source assets and create a draft; never publish a release.

The independent descriptor keeps the published v0.3 release and tag untouched.
Only an explicit descriptor change triggers the workflow. Ordinary source
commits do not rebuild or replace its downloads. The upload phase has no publish
flag, tag-force operation or fallback to the existing public release.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import subprocess

import package_release
import publish_release

ROOT = Path(__file__).resolve().parents[2]
PROFILE = 'mod/draft.json'


def descriptor(root: Path) -> dict:
    draft = json.loads((root / PROFILE).read_text(encoding='utf-8'))
    development = json.loads((root / 'mod/development.json').read_text(encoding='utf-8'))
    published = json.loads((root / 'mod/release.json').read_text(encoding='utf-8'))
    if (draft.get('draft') is not True or draft.get('publish') is not False
            or draft.get('tag') == published['tag']
            or not re.fullmatch(r'v\d+\.\d+(?:\.\d+)?-[a-zA-Z0-9][a-zA-Z0-9.-]*', draft['tag'])
            or any(draft.get(key) != development.get(key)
                   for key in ('patch', 'config_capability', 'launcher', 'runtime_base', 'public_mod_version'))):
        raise ValueError('Expected a separate, unpublished draft matching current development.')
    notes = (root / draft['notes']).resolve()
    if not notes.is_relative_to(root.resolve()) or not notes.is_file():
        raise ValueError('Draft notes must be a tracked file inside this checkout.')
    return draft


def existing_draft(releases: list[dict], tag: str, commit: str) -> dict | None:
    matches = [item for item in releases if item['tag_name'] == tag]
    if len(matches) > 1:
        raise ValueError('More than one release uses the requested draft tag.')
    if not matches:
        return None
    release = matches[0]
    if release.get('draft') is not True or release.get('target_commitish') != commit:
        raise ValueError('Refusing to change a published release or a draft from another commit.')
    return release


def upload(root: Path, assets: Path) -> str:
    draft = descriptor(root)
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
    repo = os.environ.get('GH_REPO', '')
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repo):
        raise ValueError('Set GH_REPO to the authorized owner/repository.')
    _, files = publish_release.validate_player(root, assets, commit, PROFILE)
    files += publish_release.source_archive(root, assets, draft['tag'], commit)
    # List all releases, including private drafts, before any mutation. Do not
    # interpret an API failure as a missing release.
    pages = json.loads(subprocess.check_output(
        ['gh', 'api', '--paginate', '--slurp', f'repos/{repo}/releases?per_page=100'], text=True))
    existing = existing_draft([item for page in pages for item in page], draft['tag'], commit)
    notes = root / draft['notes']
    if existing is None:
        subprocess.run(['gh', 'release', 'create', draft['tag'], *map(str, files), '--draft',
                        '--target', commit, '--title', draft['title'], '--notes-file', str(notes)],
                       cwd=root, check=True)
    else:
        # Idempotent reruns are allowed only for this exact commit's still-private
        # draft. Never pass --clobber until that identity has been checked.
        subprocess.run(['gh', 'release', 'upload', draft['tag'], *map(str, files), '--clobber'],
                       cwd=root, check=True)
    state = publish_release.api(repo, f'releases/tags/{draft["tag"]}')
    existing_draft([state], draft['tag'], commit)
    publish_release.verify_uploaded(repo, draft['tag'], files)
    print('Saved unpublished draft: ' + state['html_url'], flush=True)
    return state['html_url']


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=('package', 'upload'))
    parser.add_argument('--assets', type=Path, default=ROOT / 'release-assets')
    args = parser.parse_args()
    descriptor(ROOT)
    if args.stage == 'package':
        package_release.main(PROFILE)
    else:
        upload(ROOT, args.assets)


if __name__ == '__main__':
    main()
