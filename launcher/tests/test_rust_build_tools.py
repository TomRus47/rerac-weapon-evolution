"""Broken installed Rust recovers locally before Cargo or installed files change."""
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools/mod'))
import rust_build_tools as tools

class RustBuildToolsTests(unittest.TestCase):
    def fixture(self,folder):
        # Windows TEMP may use an 8.3 alias; production resolves kit roots.
        root=Path(folder).resolve();source=root/'source';source.mkdir();bundle=root/'kit';bundle.mkdir()
        return source,bundle

    def test_working_global_compiler_needs_no_download(self):
        with tempfile.TemporaryDirectory() as folder,patch.dict(os.environ,{'PATH':'original'},clear=True):
            source,bundle=self.fixture(folder)
            with patch.object(tools.shutil,'which',return_value='/installed/rustup.exe'), \
                 patch.object(tools,'checked_environment',return_value=({'PATH':'verified','RUSTC':'real-rustc'},'rustc verified')), \
                 patch.object(tools,'install_toolchain') as install:
                tools.ensure_rust_compiler(source,bundle)
                install.assert_not_called();self.assertEqual(os.environ['RUSTC'],'real-rustc')
                self.assertFalse((bundle/'build-tools').exists())

    def test_missing_global_compiler_installs_in_separate_root_and_keeps_cargo_cache(self):
        with tempfile.TemporaryDirectory() as folder,patch.dict(os.environ,{'PATH':'original','CARGO_HOME':'existing-cache'},clear=True):
            source,bundle=self.fixture(folder)
            def check(rustup,env,source):
                if 'RUSTUP_HOME' not in env:raise ValueError('installed component has no executable')
                return dict(env),'rustc recovered'
            with patch.object(tools.shutil,'which',return_value='/installed/rustup.exe'), \
                 patch.object(tools,'checked_environment',side_effect=check), \
                 patch.object(tools,'install_toolchain') as install:
                tools.ensure_rust_compiler(source,bundle)
                self.assertEqual(install.call_args.args[1],tools.CHANNEL)
                self.assertEqual(install.call_args.args[2]['CARGO_HOME'],'existing-cache')
                selected=Path(os.environ['RUSTUP_HOME']);self.assertEqual(selected.parent,bundle/'build-tools')
                self.assertEqual(os.environ['CARGO_HOME'],'existing-cache')
                self.assertEqual(json.loads((bundle/'build-tools/verified-rust.json').read_text())['root'],selected.name)

    def test_verified_local_cache_is_probed_and_reused(self):
        with tempfile.TemporaryDirectory() as folder,patch.dict(os.environ,{'PATH':'original'},clear=True):
            source,bundle=self.fixture(folder);cache=bundle/'build-tools';cache.mkdir()
            (cache/'verified-rust.json').write_text('{"root":"rustup-123"}')
            def check(rustup,env,source):
                if 'RUSTUP_HOME' not in env:raise OSError('Access denied')
                return dict(env),'rustc cached'
            with patch.object(tools.shutil,'which',return_value='/installed/rustup.exe'), \
                 patch.object(tools,'checked_environment',side_effect=check), \
                 patch.object(tools,'install_toolchain') as install:
                tools.ensure_rust_compiler(source,bundle);install.assert_not_called()
                self.assertEqual(Path(os.environ['RUSTUP_HOME']),cache/'rustup-123')

    def test_sibling_test_kit_verified_compiler_is_reused_without_download(self):
        with tempfile.TemporaryDirectory() as folder,patch.dict(os.environ,{'PATH':'original'},clear=True):
            source,bundle=self.fixture(folder)
            for i in range(40):(bundle.parent/f'ReRAC_Weapon_Evolution_v999_{i}').mkdir()
            old=bundle.parent/'ReRAC_Weapon_Evolution_v117_3_Download_Recovery'/'build-tools'
            old.mkdir(parents=True);(old/'verified-rust.json').write_text('{"root":"rustup-1173"}')
            def check(rustup,env,source):
                if env.get('RUSTUP_HOME')!=str(old/'rustup-1173'):raise ValueError('not the working cache')
                return dict(env),'rustc verified sibling'
            with patch.object(tools.shutil,'which',return_value='/installed/rustup.exe'), \
                 patch.object(tools,'checked_environment',side_effect=check), \
                 patch.object(tools,'install_toolchain') as install:
                tools.ensure_rust_compiler(source,bundle);install.assert_not_called()
                self.assertEqual(Path(os.environ['RUSTUP_HOME']),old/'rustup-1173')

    def test_blocked_new_compiler_stops_and_does_not_publish_ready_cache(self):
        with tempfile.TemporaryDirectory() as folder,patch.dict(os.environ,{'PATH':'original'},clear=True):
            source,bundle=self.fixture(folder)
            with patch.object(tools.shutil,'which',return_value='/installed/rustup.exe'), \
                 patch.object(tools,'checked_environment',side_effect=PermissionError('Access denied')), \
                 patch.object(tools,'install_toolchain'):
                with self.assertRaisesRegex(ValueError,'separate Rust compiler also failed'):
                    tools.ensure_rust_compiler(source,bundle)
                self.assertEqual(dict(os.environ),{'PATH':'original'})
                self.assertFalse((bundle/'build-tools/verified-rust.json').exists())

    def test_probe_requires_real_metadata_output(self):
        with patch.object(tools,'run',side_effect=['rustc test','']):
            with self.assertRaisesRegex(ValueError,'probe metadata'):
                tools.probe(Path('/rustc'),{},ROOT)

    def test_real_binary_paths_replace_case_insensitive_environment_keys(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);compiler=root/'rustc.exe';cargo=root/'cargo.exe'
            compiler.touch();cargo.touch()
            with patch.object(tools,'run',side_effect=[str(compiler),str(cargo),'cargo verified']), \
                 patch.object(tools,'probe',return_value='rustc verified'):
                env,version=tools.checked_environment('/rustup',{'Path':'previous','rustc':'broken'},root)
                self.assertNotIn('Path',env);self.assertNotIn('rustc',env)
                self.assertEqual(env['RUSTC'],str(compiler));self.assertTrue(env['PATH'].endswith('previous'))
                self.assertEqual(version,'rustc verified')

    def test_recovery_is_called_before_build_engine_in_entrypoint(self):
        source=(ROOT/'tools/mod/runtime038_test.py').read_text()
        self.assertLess(source.index('ensure_rust_compiler(SOURCE,BUNDLE)'),source.index('exe = build_engine('))

if __name__=='__main__':unittest.main()
