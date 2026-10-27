# Weapon Evolution v102: combat feedback and HUD

Development patch **0.1.6.102**, configuration capability **88**, launcher **0.28.7**, for **ReRAC 0.3.6-alpha**. Public mod version remains **0.3**. This is a source/main update and local test kit, not a new published release. The approved balance is retained; the author will supply a new default profile after testing.

Weapon Evolution is an unofficial, independent mod by TomRus47. The mod author does not develop ReRAC or the original games, owns no original-game rights or assets, and is not affiliated with their rights holders. Upstream licenses and credits remain applicable.

## Install the local test patch

1. Extract the entire test ZIP into a separate folder. Close the game and mod launcher.
2. Run **01_UPDATE_BUILD_AND_OPEN.bat**.
3. Select the existing **Rust source folder**, containing Cargo.toml and crates. Select the existing **playable mod folder**, containing the launcher EXE, weapons.ini and runtime/rerac.exe. If requested, select the current launcher source folder containing launcher.py and build_launcher_exe.py. Do not select this update folder as an existing installation.
4. Rust and Visual Studio x64 C++ tools must be installed. The helper initializes the installed compiler environment automatically, tests and builds the game, rebuilds the launcher and replaces both playable EXEs. The launcher title must show **0.28.7**.
5. A backup is retained inside the playable folder. Failed builds restore source, profiles, icon and playable EXEs. The updater accepts only the reviewed supplied v101 source/launcher baseline or its own already-updated files; unknown local code is rejected before writes. Personal INI values, presets and saves remain local.

To make a standalone player ZIP after testing, run **02_CREATE_PLAYABLE_ZIP.bat**. It uses the installed runtime/launcher and CURRENT weapons.ini, omits author presets and produces the folder, ZIP and checksum under the playable folder's dist directory. This is a local packaging action and does not upload or publish anything. Players extract the complete package and open its EXE; game data must be provided separately. Do not import the standalone ZIP into the official ReRAC launcher.

## Automatic combat feedback

Committed enemy HP loss refreshes a red hit tint. The tint fades over 15 native-reference ticks after the last hit; sustained accepted damage refreshes it. It does not restart stagger, alter invulnerability, change HP or award extra XP. Native creature classes and known donor/boss adapters use the same visual path.

Pyrocitor, its lava mode, and flame-mine damage also arm a finite native burn-puff cycle. Actual stream contact renews an already accepted burn; a queued or shield-rejected contact alone does not ignite a target. A batch is not emitted every tick. Its longest particle life expires before another batch is allowed, and only continued fire contact restarts it. Native body burn emission is suppressed while this independent cycle owns it. Each victim retains the attacking weapon's burn palette; mine and Pyrocitor colors remain independent. Fire stops renewing when contact ends.

`moby_update/creature/damage.rs` is the accepted-loss boundary. `moby_update/feedback.rs` owns the short-lived visual state. `scheduler.rs` restores the previous tint before native updates and applies the next tint after the frame. This avoids tint accumulation in native flash records. Actor identity includes class and UID; deletion/reuse and level resets discard stale effects. Feedback processing is sorted by actor ID to keep particle RNG order stable.

## Per-grade OmniWrench controls

All eight grades receive these standard fields, Off by default:

```ini
burn_on_hit = false
hud_quickselect_only = false
```

- **Ignite struck enemies (visual only):** accepted combo, airborne, thrown/outbound/return and slam hits add the same finite fire effect. It adds no burn damage. The flag is captured at attack/throw start.
- **Show wrench name and XP only after Triangle:** requires the existing Show HUD option to be On. The name, vignette and weapon XP use the Triangle-triggered Nanotech HUD visibility lease and fade; damage/healing alone cannot arm it. Swimming hides every custom weapon name/XP display regardless of this setting.

The experimental frame, after every standard field, adds:

```ini
slam_explosion_enabled = false
slam_explosion_damage_hp = 200
slam_explosion_radius = 0.5
slam_explosion_color = default
```

A ground-slam impact consumes its pending impact once and adds a bomb-style radial explosion. Its HP, radius and palette are captured at attack entry. The normal slam retains its own damage. The explosion uses bomb hit/reaction identity while routing actual-loss XP to the wrench. Direct slam and explosion have distinct per-instance keys; each explosion damages a given enemy once, and the next slam can hit again immediately. It cannot damage Ratchet. The same Bomb Glove visual recipe is used with an explicit independent palette; no Bomb Glove settings are changed. Radius is in world units; 0.5 matches the ordinary native bomb radius. Color is default or RRGGBB.

This is an additional attack enhancement configured by the launcher/INI, not an R1-selectable firing mode. The unsupported wrench R1 selector stays hidden. Its saved zero value is retained for compatibility. Other weapons' R1 modes are unchanged.

## HUD

In **Global Settings -> XP Bars and settings**:

```ini
[hud]
weapon_bar_alternative_position = false
```

On places the weapon bar between the existing name panel and the ammunition panel. The ammo panel moves down by bar height plus four HUD units to leave a real gap. Name placement stays at its original top edge. The bar's outside edges still follow the ammunition vignette, including widescreen and slide/fade geometry. Off retains the bar below ammunition. Supported bar heights remain 2–16.

Weapon-name text uses the native ammunition indigo for V1–V4, yellow for V5–V7 and white for V8, with the existing opacity/fade. This does not change the weapon-icon grade palette or Nanotech bar layout.

## Randomizer models and NG+

Rewards without their own inventory model use the native **Gadgetron ammo-crate mesh (class 511)**. Real Gold Bolt rewards retain class 1134, coordinates retain the upright Infobot, and existing modeled items retain their own meshes. The loader imports the crate's geometry/textures before gameplay. The crate is a static standalone display, not an initialized crate actor: no collision, break, ammo drop or mission script is added. Its halo, reward text, collection/jingle and disappearance are the same as other randomized rewards. Orientation and sequence are fixed to the native upright display pose. Existing saves/seeds work unchanged.

The supplied v97 route catalog and v100/v101 fixes are synchronized into main with this development update, including independent Thruster-Pack moves, discovered-weapon ammo policy and access-only Gemlik/explosive safeguards. Existing saved placements never reroll.

Extra enemies are enabled in NG+ as well as ordinary runs. **Extra enemies per level** is an exact integer budget, not a coefficient: 5 still means five additional eligible actors with NGPlus_Multiplier set to 2. That multiplier scales the existing health/damage/motion/perception coefficients. Native base HP continues to decide the duplication category. A regression exercises five medium-class copies with NG+ and doubled difficulty.

## Verification and remaining playtests

Run the game library tests, public API tests, launcher tests and `tools/mod/check_source.py`. `launcher/tests/test_feedback_hud.py` covers per-grade controls, independent values, ordering, localization, migration and runtime capability. The frozen Windows self-test renders and toggles the real Off/On controls for all eight wrench grades before the installer accepts the new EXE.

Host tests validate data/logic and engine Rust types, not visual quality or a Windows link. Test Pyrocitor/lava/mines on multiple enemy types and bosses, stop/start fire, switch weapons during a burn, die/reload, and verify bounded particles. Check wrench throws/combos/slams and XP, swimming, Triangle-only visibility, both bar positions at multiple heights, NG+ duplicates, and visible crate/Gold Bolt/Infobot models in both world and vendor displays. Approve the new default profile before creating a public release.
