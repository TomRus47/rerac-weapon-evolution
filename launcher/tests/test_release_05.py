"""0.5 publishes only a verified player ZIP; native tagged archives supply source."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/mod'))
import release_05 as p


class Release05Tests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name); self.assets = self.root / 'assets'; self.assets.mkdir()
        for name in ('mod', 'docs', 'launcher'):(self.root / name).mkdir()
        self.release = {'tag':'v0.5', 'title':'Weapon Evolution 0.5', 'public_mod_version':'0.5',
            'runtime_base':'0.3.8-alpha', 'patch':'0.1.6.140', 'config_capability':97, 'launcher':'0.30.2',
            'asset':'ReRAC_Weapon_Evolution_0.5_windows_x64', 'publish':True,
            'download_policy':'windows-and-native-source', 'notes':'docs/notes.md'}
        (self.root / p.PROFILE).write_text(json.dumps(self.release))
        (self.root / 'mod/development.json').write_text(json.dumps(self.release))
        (self.root / 'docs/notes.md').write_text('English approved release notes')
        (self.root / 'launcher/weapons.default.ini').write_bytes(b'approved profile')
        (self.root / 'launcher/launcher.ico').write_bytes(b'icon')
        self.archive = self.assets / (self.release['asset'] + '.zip')
        self.prefix = self.release['asset'] + '/'; self.commit = 'a' * 40
        with zipfile.ZipFile(self.archive, 'w') as z:
            for n, b in {'ReRAC_Weapon_Evolution.exe':b'GUI', 'runtime/rerac.exe':b'game',
                         'weapons.ini':b'approved profile','LEGGIMI.txt':b'English player instructions'}.items():
                z.writestr(self.prefix+n,b)
        self.report = {'release':'v0.5', 'commit':self.commit, 'launcher':'0.30.2',
            'archive':self.archive.name, 'sha256':p.publish_release.digest(self.archive),
            'default_ini_sha256':p.publish_release.digest(self.root/'launcher/weapons.default.ini'),
            'icon_sha256':p.publish_release.digest(self.root/'launcher/launcher.ico'),
            'runtime':{'version':'0.3.8','weapon_evolution_patch':'0.1.6.140',
                       'weapon_evolution_config':97,'weapon_evolution_audio_diagnostics':False}}
        (self.assets/'release-build.json').write_text(json.dumps(self.report))
        Path(str(self.archive)+'.sha256').write_text(self.report['sha256']+'  '+self.archive.name+'\n')
        self.state = {'id':1,'tag_name':'v0.5','target_commitish':self.commit,'draft':True,'prerelease':False,
            'html_url':'https://github.com/TomRus47/rerac-weapon-evolution/releases/tag/v0.5',
            'assets':[{'name':self.archive.name,'state':'uploaded','digest':'sha256:'+self.report['sha256']}]}

    def test_descriptor_requires_the_explicit_matching_05_authorization(self):
        self.assertEqual(p.descriptor(self.root),self.release)
        for changes in ({'publish':False},{'tag':'v0.6'},{'patch':'0.1.6.123'},
                        {'download_policy':'all-artifacts'},{'notes':'../outside.md'}):
            (self.root/p.PROFILE).write_text(json.dumps(self.release|changes))
            with self.assertRaises(ValueError):p.descriptor(self.root)

    def test_provenance_is_inside_the_player_and_sidecar_hashes_follow_final_bytes(self):
        with patch('migrate_ini.convert',return_value=(b'approved profile',[])):
            p.finalize_player(self.root,self.assets,self.commit)
        with zipfile.ZipFile(self.archive) as z:
            self.assertIn(self.prefix+'README_FIRST.txt',z.namelist())
            self.assertNotIn(self.prefix+'LEGGIMI.txt',z.namelist())
            self.assertEqual(z.read(self.prefix+'weapons.ini'),b'approved profile')
            info=json.loads(z.read(self.prefix+'BUILD_INFO.json'))
            self.assertEqual(info['commit'],self.commit);self.assertEqual(info['mod_version'],'0.5')
        report=json.loads((self.assets/'release-build.json').read_text())
        self.assertEqual(report['sha256'],p.publish_release.digest(self.archive))
        p.publish_release.validate_player(self.root,self.assets,self.commit,p.PROFILE)

    def test_extra_attachments_or_mismatched_uploaded_bytes_cannot_publish(self):
        p.validate_remote(self.release,self.archive,self.state,self.commit)
        for bad in (self.state|{'assets':self.state['assets']+[{'name':'release-build.json'}]},
                    self.state|{'target_commitish':'b'*40},
                    self.state|{'assets':[self.state['assets'][0]|{'digest':'sha256:'+'0'*64}]}):
            with self.assertRaises(ValueError):p.validate_remote(self.release,self.archive,bad,self.commit)

    def test_already_public_or_foreign_draft_release_is_never_mutated(self):
        for state in (self.state|{'draft':False},self.state|{'target_commitish':'b'*40}):
            with patch.dict(os.environ,GH_REPO=p.REPO,GITHUB_SHA=self.commit), \
                 patch.object(p.subprocess,'check_output',side_effect=[self.commit,json.dumps([[state]])]), \
                 patch.object(p.publish_release,'source_archive'), patch.object(p.subprocess,'run') as run, \
                 patch.object(p,'api') as api:
                with self.assertRaises(ValueError):p.upload(self.assets,self.root)
                run.assert_not_called();api.assert_not_called()

    def test_only_verified_windows_zip_upload_precedes_publication(self):
        published=self.state|{'draft':False}
        calls=[None,{},self.state,{'object':{'sha':self.commit}},published]
        with patch.dict(os.environ,GH_REPO=p.REPO,GITHUB_SHA=self.commit), \
             patch.object(p.subprocess,'check_output',side_effect=[self.commit,'[[]]',json.dumps([[self.state]])]), \
             patch.object(p.publish_release,'source_archive') as source, \
             patch.object(p.subprocess,'run') as run, patch.object(p,'api',side_effect=calls):
            p.upload(self.assets,self.root)
            create=run.call_args_list[0].args[0]
            self.assertEqual(create[:4],['gh','release','create','v0.5'])
            self.assertIn(str(self.archive),create);self.assertIn('--draft',create)
            self.assertFalse(any(str(x).endswith(('.sha256','release-build.json','_source.zip')) for x in create))
            self.assertIn('--draft=false',run.call_args_list[-1].args[0])
            source.assert_called_once()

    def test_wrong_upload_digest_stops_before_publication(self):
        bad=self.state|{'assets':[self.state['assets'][0]|{'digest':'sha256:'+'0'*64}]}
        with patch.dict(os.environ,GH_REPO=p.REPO,GITHUB_SHA=self.commit), \
             patch.object(p.subprocess,'check_output',side_effect=[self.commit,'[[]]',json.dumps([[self.state]])]), \
             patch.object(p.publish_release,'source_archive'),patch.object(p.subprocess,'run') as run, \
             patch.object(p,'api',side_effect=[None,{},bad]):
            with self.assertRaises(ValueError):p.upload(self.assets,self.root)
            self.assertEqual(run.call_count,1);self.assertIn('--draft',run.call_args.args[0])


if __name__=='__main__':unittest.main()
