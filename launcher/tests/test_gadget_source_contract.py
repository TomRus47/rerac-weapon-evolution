"""Cross-crate integration wiring complements native gameplay regressions."""
from pathlib import Path
import re,sys,unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/mod'))
class IntegrationTests(unittest.TestCase):
 def test_runtime_identity_and_complete_prepared_interfaces(self):
  from verified_artifacts import validate_source_apis
  import runtime038_test
  validate_source_apis(ROOT)
  runtime038_test.validate_identity({'name':'rerac','game':'rac1','version':'0.3.8',
      'weapon_evolution_patch':'0.1.6.142','weapon_evolution_config':97,'weapon_evolution_audio_diagnostics':False})
 def test_gadget_hooks_are_in_shared_tick_not_planet_specific_scripts(self):
  tick=(ROOT/'crates/rc-game/src/tick.rs').read_text()
  self.assertIn('crate::gadget_combat::item_tick(',tick)
  self.assertLess(tick.index('crate::gadget_combat::item_tick('),tick.index('crate::hero::gadgets::flush_item_sounds'))
  scheduler=(ROOT/'crates/rc-game/src/moby_update/scheduler.rs').read_text()
  self.assertIn('!gadget_frozen &&',scheduler)
  self.assertIn('super::creature::damage::frozen_damage',scheduler)
 def test_frost_ammo_is_not_a_native_weapon_shop_record(self):
  module=(ROOT/'crates/rc-game/src/gadget_combat.rs').read_text()
  self.assertNotIn('weapons.ammo[22]',module)
  engine=(ROOT/'crates/rc-engine/src/gameplay.rs').read_text()
  self.assertIn('hydro_combat_ammo',engine)
if __name__=='__main__':unittest.main()
