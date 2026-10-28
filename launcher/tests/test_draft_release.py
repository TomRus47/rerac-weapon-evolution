"""A candidate build may create a private draft, never replace a public release."""
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/mod'))
spec = importlib.util.spec_from_file_location('draft_release', ROOT / 'tools/mod/draft_release.py')
draft = importlib.util.module_from_spec(spec)
spec.loader.exec_module(draft)


class DraftReleaseTests(unittest.TestCase):
    def test_descriptor_separates_the_public_tag_and_requires_current_identity(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'mod').mkdir()
            (root / 'docs').mkdir()
            (root / 'docs/notes.md').write_text('Draft')
            current = json.loads((ROOT / 'mod/development.json').read_text())
            record = json.loads((ROOT / 'mod/draft.json').read_text()) | {'notes': 'docs/notes.md'}
            (root / 'mod/development.json').write_text(json.dumps(current))
            (root / 'mod/release.json').write_text('{"tag":"v0.3"}')
            path = root / 'mod/draft.json'
            path.write_text(json.dumps(record))
            self.assertEqual(draft.descriptor(root), record)
            for changes in ({'draft': False}, {'publish': True}, {'tag': 'v0.3'},
                            {'patch': '0.1.6.59'}, {'notes': '../outside.md'}):
                path.write_text(json.dumps(record | changes))
                with self.assertRaises(ValueError):
                    draft.descriptor(root)

    def test_published_or_different_commit_release_is_never_reused(self):
        record = {'tag_name': 'v0.3-next', 'draft': True, 'target_commitish': 'a' * 40}
        self.assertEqual(draft.existing_draft([record], 'v0.3-next', 'a' * 40), record)
        self.assertIsNone(draft.existing_draft([], 'v0.3-next', 'a' * 40))
        for record in (record | {'draft': False}, record | {'target_commitish': 'b' * 40}):
            with self.assertRaises(ValueError):
                draft.existing_draft([record], 'v0.3-next', 'a' * 40)

    def test_upload_is_draft_only_and_includes_exact_source(self):
        record = draft.descriptor(ROOT)
        commit = 'a' * 40
        files = [Path('player.zip'), Path('source.zip')]
        state = {'tag_name': record['tag'], 'draft': True,
                 'target_commitish': commit, 'html_url': 'https://github.com/example/draft'}
        with patch.dict(os.environ, GH_REPO='owner/repo'), \
                patch.object(draft.subprocess, 'check_output', side_effect=[commit, '[[]]']), \
                patch.object(draft.subprocess, 'run') as run, \
                patch.object(draft.publish_release, 'validate_player', return_value=(record, files[:1])) as validate, \
                patch.object(draft.publish_release, 'source_archive', return_value=files[1:]) as archive, \
                patch.object(draft.publish_release, 'api', return_value=state), \
                patch.object(draft.publish_release, 'verify_uploaded') as verify:
            self.assertEqual(draft.upload(ROOT, Path('assets')), state['html_url'])
            command = run.call_args.args[0]
            self.assertEqual(command[:3], ['gh', 'release', 'create'])
            self.assertIn('--draft', command)
            self.assertNotIn('--draft=false', command)
            self.assertIn('source.zip', command)
            self.assertIn(commit, command)
            archive.assert_called_once_with(ROOT, Path('assets'), record['tag'], commit)
            self.assertEqual(validate.call_args.args[-1], 'mod/draft.json')
            verify.assert_called_once()

    def test_release_list_failure_does_not_create_anything(self):
        with patch.dict(os.environ, GH_REPO='owner/repo'), \
                patch.object(draft.subprocess, 'check_output', side_effect=['a' * 40, RuntimeError('API unavailable')]), \
                patch.object(draft.subprocess, 'run') as run, \
                patch.object(draft.publish_release, 'validate_player', return_value=({}, [])), \
                patch.object(draft.publish_release, 'source_archive', return_value=[]):
            with self.assertRaises(RuntimeError):
                draft.upload(ROOT, Path('assets'))
            run.assert_not_called()
