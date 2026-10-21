"""Release replacement must validate identities and uploads before moving v0.3."""
from pathlib import Path
from unittest.mock import patch
import hashlib, importlib.util, json, tempfile, unittest, zipfile
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('publish_release',ROOT/'tools/mod/publish_release.py')
p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)

class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.assets=self.root/'assets';self.assets.mkdir()
        self.release={'tag':'v0.3','title':'Weapon Evolution 0.3','asset':'player','patch':'0.1.6.59',
                      'config_capability':58,'launcher':'0.17.2','replace_tag_sha':'a'*40}
        (self.root/'mod').mkdir();(self.root/'launcher').mkdir()
        (self.root/'mod/release.json').write_text(json.dumps(self.release))
        (self.root/'launcher/weapons.default.ini').write_bytes(b'profile')
        (self.root/'launcher/launcher.ico').write_bytes(b'icon')
        self.archive=self.assets/'player.zip'
        with zipfile.ZipFile(self.archive,'w') as z:z.writestr('runtime/rerac.exe',b'tested Windows game')
        self.sidecar=Path(str(self.archive)+'.sha256');self.sidecar.write_text(p.digest(self.archive)+'  player.zip\n')
        self.report={'release':'v0.3','commit':'b'*40,'launcher':'0.17.2','archive':'player.zip','sha256':p.digest(self.archive),
                     'default_ini_sha256':p.digest(self.root/'launcher/weapons.default.ini'),'icon_sha256':p.digest(self.root/'launcher/launcher.ico'),
                     'runtime':{'weapon_evolution_patch':'0.1.6.59','weapon_evolution_config':58,'weapon_evolution_audio_diagnostics':False,'version':'0.3.4'}}
        self.report_path=self.assets/'release-build.json';self.report_path.write_text(json.dumps(self.report))
        self.files=[self.archive,self.sidecar,self.report_path]
        self.remote={'assets':[{'name':f.name,'state':'uploaded','digest':'sha256:'+p.digest(f)} for f in self.files]}

    def test_player_archive_report_profile_and_icon_must_match_exact_commit(self):
        release,files=p.validate_player(self.root,self.assets,'b'*40)
        self.assertEqual(release,self.release);self.assertEqual(files,self.files)
        for field,value in [('commit','c'*40),('sha256','0'*64),('launcher','0.16.0'),('icon_sha256','0'*64)]:
            with self.subTest(field=field):
                self.report_path.write_text(json.dumps(self.report|{field:value}))
                with self.assertRaises(ValueError):p.validate_player(self.root,self.assets,'b'*40)

    def test_changed_tag_rejects_before_upload_and_bad_uploaded_digest_prevents_tag_change(self):
        with patch.object(p,'api',return_value={'object':{'sha':'c'*40}}),patch.object(p.subprocess,'run') as run:
            with self.assertRaisesRegex(ValueError,'tag changed'):p.replace_release('owner/repo',self.release,self.files,'b'*40,Path('notes.md'))
            run.assert_not_called()
        bad={'assets':[entry|{'digest':'sha256:'+'0'*64} for entry in self.remote['assets']]}
        with patch.object(p,'api',side_effect=[{'object':{'sha':'a'*40}},bad]),patch.object(p.subprocess,'run') as run:
            with self.assertRaisesRegex(ValueError,'digest differs'):p.replace_release('owner/repo',self.release,self.files,'b'*40,Path('notes.md'))
            self.assertEqual(run.call_count,1);self.assertIn('--clobber',run.call_args.args[0]);self.assertNotIn('PATCH',run.call_args.args[0])

    def test_verified_assets_replace_same_release_and_move_tag_to_matching_source(self):
        responses=[{'object':{'sha':'a'*40}},self.remote,{'object':{'sha':'a'*40}},{'object':{'sha':'b'*40}}]
        with patch.object(p,'api',side_effect=responses),patch.object(p.subprocess,'run') as run:
            p.replace_release('owner/repo',self.release,self.files,'b'*40,Path('notes.md'))
            self.assertEqual(run.call_count,3)
            patch_call=run.call_args_list[1]
            self.assertIn('PATCH',patch_call.args[0]);self.assertEqual(json.loads(patch_call.kwargs['input'])['sha'],'b'*40)
            self.assertEqual(run.call_args_list[2].args[0][:4],['gh','release','edit','v0.3'])
            self.assertIn('--draft=false',run.call_args_list[2].args[0])

    def test_other_public_version_never_replaces_any_asset(self):
        with patch.object(p.subprocess,'run') as run:
            with self.assertRaises(ValueError):p.replace_release('owner/repo',self.release|{'tag':'v0.4'},self.files,'b'*40,Path('notes.md'))
            run.assert_not_called()
