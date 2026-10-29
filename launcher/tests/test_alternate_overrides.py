"""Per-grade mode-only tuning, lossless defaults and runtime capability guards."""
from pathlib import Path
import sys,tempfile,unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import launcher_core as core
import migrate_ini
import schema
import i18n

KEYS={'pyrocitor':['lava_damage_bonus_percent'],
      'mine_glove':['flame_mine_max_active_mines','flame_mine_lifetime_seconds']}
class AlternateOverrideTests(unittest.TestCase):
    def test_migration_inherits_personal_standard_mine_values_and_is_idempotent(self):
        raw=Path(__file__).resolve().parents[1].joinpath('weapons.default.ini').read_bytes()
        raw=b''.join(line for line in raw.splitlines(True) if line.split(b'=',1)[0].strip().decode() not in sum(KEYS.values(),[]))
        before=core.checked_document(raw)
        edits={}
        for n in range(1,9):
            section=f'weapon.mine_glove.level{n}'
            edits[section,'max_active_mines']=str(n)
            edits[section,'mine_lifetime_seconds']=str(n+10.5)
        custom=before.render(edits)
        custom=b'\xef\xbb\xbf'+custom.replace(b'\n',b'\r\n')+b'; keep my note\r\n'
        after=core.profile_document(custom)
        for pair,value in core.checked_document(custom).entries.items():self.assertEqual(after.entries[pair],value,pair)
        for n in range(1,9):
            section=f'weapon.mine_glove.level{n}'
            self.assertEqual(after.entries[section,'flame_mine_max_active_mines'],str(n))
            self.assertEqual(after.entries[section,'flame_mine_lifetime_seconds'],str(n+10.5))
            self.assertEqual(after.entries[f'weapon.pyrocitor.level{n}','lava_damage_bonus_percent'],'25')
        self.assertIn(b'; keep my note\r\n',after.raw)
        self.assertEqual(core.profile_document(after.raw).raw,after.raw)
        self.assertEqual(core.required_config(before),92)
        self.assertEqual(core.required_config(after),92)
    def test_independent_explicit_overrides_survive_preset_and_session_export(self):
        doc=core.profile_document(Path(__file__).resolve().parents[1].joinpath('weapons.default.ini').read_bytes())
        edits={}
        for n in range(1,9):
            edits[f'weapon.pyrocitor.level{n}','lava_damage_bonus_percent']=str(n*12.5)
            edits[f'weapon.mine_glove.level{n}','flame_mine_max_active_mines']=str(n+8)
            edits[f'weapon.mine_glove.level{n}','flame_mine_lifetime_seconds']=str(n+2.5)
        raw=doc.render(edits)
        with tempfile.TemporaryDirectory() as temporary:
            store=core.PresetStore(Path(temporary));store.save('Overrides',raw)
            loaded=store.read('Overrides');runtime=core.checked_document(migrate_ini.runtime_profile(loaded.raw))
            for pair,value in edits.items():self.assertEqual(loaded.entries[pair],value);self.assertEqual(runtime.entries[pair],value)
        for weapon,keys in KEYS.items():
            for n in range(1,9):
                section=f'weapon.{weapon}.level{n}'
                standard,experimental=schema.field_groups(weapon,loaded.keys(section))
                for key in keys:self.assertNotIn(key,standard);self.assertIn(key,experimental)
                colors=[k for k in experimental if k.endswith('_color')]
                for key in keys:
                    for color in colors:self.assertLess(experimental.index(key),experimental.index(color))
    def test_limits_and_localized_metadata_cover_all_languages(self):
        cases=[('pyrocitor','lava_damage_bonus_percent',['0','25','1000'],['-1','1001','NaN']),
               ('mine_glove','flame_mine_max_active_mines',['1','12','32'],['0','33','2.5']),
               ('mine_glove','flame_mine_lifetime_seconds',['0.5','12.5','600'],['0','601','inf'])]
        for weapon,key,valid,invalid in cases:
            section=f'weapon.{weapon}.level1'
            for value in valid:core.validate_value(section,key,value)
            for value in invalid:
                with self.assertRaises(ValueError):core.validate_value(section,key,value)
            self.assertTrue(schema.HELP[key])
            for language in i18n.LANGUAGES:
                self.assertIn(language,i18n.FIELD_LABELS[key]);self.assertIn(language,i18n.HELP_TEXT[key])
