"""The distributed kit must pass its real build preflight before Cargo runs."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/mod'))
import verified_artifacts as artifacts
import create_combat117_kit as kit

class PreparedSourceIdentityTests(unittest.TestCase):
    def fixture(self, root, version='0.1.6.117', capability=93):
        files={
            'mod/development.json':json.dumps({'patch':version,'config_capability':capability}),
            'crates/rc-engine/src/disc_source.rs':f'"weapon_evolution_patch":"{version}","weapon_evolution_config":{capability}',
            'crates/rc-game/src/weapon_evo.rs':'pub mod health_test;',
            'crates/rc-game/src/hud.rs':'pub mod nanotech; enum Draw { Nanotech16 { x:i32 } }',
            'crates/rc-game/src/weapon_evo_notice.rs':'[u16;37]',
            'crates/rc-game/src/weapon_evo_upgrade_wave.rs':'[u16;37]',
        }
        for name,text in files.items():
            p=root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text)
        return root/'crates/rc-engine/src/disc_source.rs'

    def test_current_bundled_source_passes_exact_build_preflight(self):
        artifacts.validate_source_apis(ROOT)

    def test_identity_follows_descriptor_instead_of_previous_patch_literal(self):
        for version,cap in [('0.1.6.116',92),('0.1.6.117',93),('0.1.6.118',94)]:
            with self.subTest(version=version),tempfile.TemporaryDirectory() as folder:
                root=Path(folder);self.fixture(root,version,cap)
                artifacts.validate_source_apis(root)

    def test_stale_patch_or_capability_is_rejected_before_cargo_or_mutation(self):
        for marker,value in [('0.1.6.117','0.1.6.116'),('config":93','config":92')]:
            with self.subTest(marker=marker),tempfile.TemporaryDirectory() as folder:
                root=Path(folder);runtime=self.fixture(root)
                runtime.write_text(runtime.read_text().replace(marker,value))
                with patch.object(artifacts,'workspace_packages') as cargo:
                    with self.assertRaisesRegex(ValueError,'Prepared 0.1.6.117 source is incomplete'):
                        artifacts.build_engine(root,root/'target',lambda _:None)
                    cargo.assert_not_called()

    def test_required_interfaces_and_descriptor_remain_strict(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);self.fixture(root)
            (root/'crates/rc-game/src/hud.rs').write_text('pub mod nanotech;')
            with self.assertRaisesRegex(ValueError,'Nanotech16'):
                artifacts.validate_source_apis(root)
            (root/'mod/development.json').write_text('{"patch":"wrong","config_capability":true}')
            with self.assertRaisesRegex(ValueError,'development identity'):
                artifacts.validate_source_apis(root)

    def test_export_stops_before_writing_an_invalid_kit(self):
        with tempfile.TemporaryDirectory() as folder:
            output=Path(folder)/'output'
            with patch.object(kit,'validate_source_apis',side_effect=ValueError('stale source')) as preflight:
                with self.assertRaisesRegex(ValueError,'stale source'):
                    kit.create(output)
                preflight.assert_called_once_with(kit.ROOT)
            self.assertFalse(output.exists())

if __name__=='__main__':unittest.main()
