# Developer guide

This guide is for a developer changing Weapon Evolution, rather than a player editing a preset. It describes the v0.1.6.109 gameplay implementation and v0.29.3 launcher included in this repository. The gameplay, configuration and save contracts below are implemented code; the upstream `docs/plan/mods.md` describes future upstream mod support instead.

Current model, palette and steering behavior is described in [MODELS_AND_STATIC_PAGES_V58.md](MODELS_AND_STATIC_PAGES_V58.md); its old page-cache implementation is historical. The current launcher builds all 17 page trees before showing the window, keeps each Canvas and its child form together, and coalesces scroll repaint work. See [launcher lifecycle](LAUNCHER_LIFECYCLE.md) before changing this code. The shared percentage for alternative mine models is explained in [FLAME_MINE_MODEL_SCALE.md](FLAME_MINE_MODEL_SCALE.md). See [build layout](BUILD_LAYOUT.md) and [credits](CREDITS.md).

Current additions and changed contracts are mapped in [CURRENT_IMPLEMENTATION.md](CURRENT_IMPLEMENTATION.md) and [SAVED_CONTROLS.md](SAVED_CONTROLS.md). The base is ReRAC 0.3.6-alpha; V6 is an NG+ Gold purchase for weapons, followed by V7/V8 XP. Save companions now use v9 and retain v1–v8 decoding. Older versioned examples below are historical where these current documents specify different limits.

## 1. Start here

Read [README.md](../../README.md) for build prerequisites, then [ARCHITECTURE.md](ARCHITECTURE.md) and this guide. Use [MOD_FILES.md](../../MOD_FILES.md) to locate integration files and [CONFIG_REFERENCE.md](CONFIG_REFERENCE.md) for the public INI fields accepted by the launcher. The source map identifies mod-related files within a complete ReRAC checkout; not every indexed file is a new module owned exclusively by this mod.

The normal workflow is:

1. Build the current checkout with `01_BUILD_GAME.bat` before changing it. The script tests the Rust game library, builds the game and stages `launcher/runtime/rerac.exe`; it does not require game data to compile.
2. Use `02_OPEN_LAUNCHER.bat` to create a local `launcher/weapons.ini` and run with your extracted game data. Confirm the behavior you intend to change.
3. Make the smallest relevant gameplay/configuration change and add a regression test for behavioral changes.
4. Run the focused tests, then the full game-library tests and engine build. Reproduce the scenario in the newly staged runtime.
5. Refresh export hashes if indexed source files were intentionally changed. Rebuild the standalone player ZIP only after validating the game runtime.

For source-only launcher work, `python -m unittest discover -s launcher/tests -v` needs no Rust compiler or game data. See VERIFICATION.md for the actual local results and the Windows workflow for executable build checks.

## 2. Where to change what

All paths in this table are relative to the repository root.

| Change | Start with | Related integration |
|---|---|---|
| XP thresholds, grade parameters, INI parser | `crates/rc-game/src/weapon_evo.rs`: `Evolution::parse`, `ExtraTrack`, `Stats`, `PyroStats` | Launcher schema, defaults, parser tests |
| Actual damage, attack identity, repeated-hit rules | `crates/rc-game/src/moby_update/real_hp.rs`: `Events`, `Attack`, `Origin` | `services.rs`, shared creature damage, enemy class handler |
| XP credited for an accepted hit | `crates/rc-game/src/moby_update/creature/damage.rs`: `award_health_write`, `award_effective`, `award_terminal` | `Evolution::award_damage`, `award_extra_damage`, `award_pyro_damage` |
| Enemy compatibility or boss protection | `crates/rc-game/src/moby_update/classes/units/` | `donor_class`, `terminal_class`, native damage record/header |
| Save/load or new-game progression | `crates/rc-game/src/weapon_evo_save.rs`: `Progress`, `sync_progress`, `restore_progress` | Native save integration, combat epoch reset in `World::new` |
| Target selection, reticle reach, projectile reach | `crates/rc-game/src/targeting.rs`, weapon-specific hero module | The actual projectile/beam class and effective reach calculations |
| L2 lock, R2 fire alias, release momentum | `crates/rc-game/src/hero/combat_strafe.rs` | Hero update/physics, engine input and gameplay |
| Leg direction, reverse gait, transition pose | `crates/rc-game/src/hero/combat_legs.rs`: `LegPose`, `current_run`, `clean_run_modifiers` | Animation pose layers and hero resolved movement |
| Locked camera behavior | `crates/rc-game/src/follow_camera.rs` | `combat_strafe::active`, camera input and native camera triggers |
| Weapon name, XP bar, fades, shop selection | `crates/rc-engine/src/hud_render.rs` | `crates/rc-game/src/hud.rs`, evolution HUD options |
| Launcher fields and help | `launcher/schema.py`: `EXTRA`, `BOMB`, `PYRO`, `LABELS`, `HELP` | `launcher_core.allowed_keys`, `numeric_range`, `validate_value` |
| Preset save/startup or session configuration | `launcher/launcher.py`, `launcher/launcher_core.py`, `launcher/migrate_ini.py` | Frozen process paths, private session INI, last-preset settings |
| Windows EXE or player ZIP | `launcher/build_launcher_exe.py`, `package_launcher.py`, `final_release.py` | Runtime already built by `editabile.py`; collected licenses and shaders |

The files and symbols are navigation targets, not stable public APIs. Search the callers before changing a signature.

### Weapon IDs and source modules

An inventory weapon ID is different from a projectile class ID and from a runtime moby instance index. Do not substitute one for another in hit attribution.

| Inventory ID | INI name | Main hero implementation | Projectile/effect implementation, where applicable |
|---|---|---|---|
| 9 | `suck_cannon` | `hero/suck_cannon.rs`, `hero/suck_vortex.rs` | Shared projectile/reaction paths; hit class `0x351` |
| 10 | `bomb_glove` | `hero/gloves.rs` | `moby_update/classes/bomb.rs`, `bomb_water.rs` |
| 11 | `devastator` | `hero/devastator.rs` | `moby_update/classes/devastator_missile.rs` |
| 13 | `visibomb` | `hero/visibomb.rs` | `moby_update/classes/visibomb.rs` |
| 14 | `taunter` | `hero/taunter.rs` | Attraction/stun and special-XP integration |
| 15 | `blaster` | `hero/blaster.rs` | `moby_update/classes/blaster_shot.rs` |
| 16 | `pyrocitor` | `hero/pyrocitor.rs` | Continuous contact and flame effects |
| 17 | `mine_glove` | `hero/gloves.rs` | `moby_update/classes/mine.rs` |
| 18 | `walloper` | `hero/walloper.rs` | Attack serial and shared melee delivery |
| 19 | `tesla_claw` | `hero/tesla.rs` | Continuous beam/contact delivery |
| 20 | `glove_of_doom` | `hero/gloves.rs` | `doom_canister.rs`, `doom_bot.rs` under `moby_update/classes/` |
| 21 | `morph_ray` | `hero/morph_ray.rs` | Morph reactions and chicken explosion parameters |
| 23 | `ryno` | `hero/ryno.rs` | `moby_update/classes/ryno_missile.rs` |
| 24 | `drone_device` | `hero/gloves.rs` | `moby_update/classes/drone.rs` |
| 25 | `decoy_glove` | `hero/gloves.rs` | `moby_update/classes/decoy.rs` |

All hero/class paths in the last two columns are under `crates/rc-game/src/`. Inventory ID 8 is the OmniWrench: it has a five-grade track and launcher page. `weapon_evo_wrench.rs` documents grade hooks; WEVO-v3 appends its saved track. See [v54 behavior](WRENCH_AND_PROJECTILES_V54.md). For the authoritative mapping from hit template to inventory ID, read `real_hp::weapon`; some weapons have several attack sources.

## 3. Configuration from disk to gameplay

The launch path is `profile_document` -> `runtime_profile` -> private session INI -> `game_environment` -> game process -> `Evolution::from_env` -> `Evolution::parse`.

The public profile intentionally has no damage-model switch. `runtime_profile` injects `damage_mode = real_hp` into the private session copy. `RC_WEAPON_EVOLUTION_INI` points to that absolute session path. A normal launch also supplies `RC_COMBAT_STRAFE=1`, `RC_BLASTER_SHOT_GATE_TICKS=2` and `RC_FRONTEND=1`. The source helper uses the same real-HP session conversion. Running `rerac.exe` directly with no mod environment can therefore give different behavior from running through the launcher.

`Evolution` has separate typed Bomb Glove and Pyrocitor state, plus `ExtraTrack` records for the other evolving weapons. Extra per-grade values are stored in `params[grade - 1]`. `publish_max_ammo` also publishes the current grade's extra parameters into `LEVEL_VALUES`; gameplay hooks read them through `extra`, `extra_flag` and `extra_color`. A key existing in the INI is insufficient unless the parser accepts it, the active grade publishes it and the relevant hook consumes it.

The GUI renders keys actually present in the document. Adding a key only to `schema.py` does not make it appear in an older profile. Add an explicit missing-field migration when older profiles should expose a newly introduced control. Preserve user values and comments; use the existing document/atomic-write helpers rather than serializing every profile from scratch.

Colors are RGB strings in the INI, with `default` preserving native colors. The runtime helpers convert to native packed color order and preserve the intended alpha behavior. Extra colors use an exact 24-bit value in the float registry and a default sentinel; use `extra_color`/`tint_rgba` instead of inventing a second encoding.

### Balance-only changes

For a local experiment, edit ignored `launcher/weapons.ini` or save a named preset. Existing `damage_hp`, color, XP threshold and interval settings do not require a Rust rebuild.

For a new public starting balance, edit both tracked `launcher/weapons.default.ini` and `launcher/weapons.example.ini`. Keep the optional embedded profile synchronized using the maintenance recipe below. `final_release.install_default` installs a changed approved default once, with a backup, based on its fingerprint; subsequent unchanged builds preserve manual edits. Check the selected release input before packaging.

XP thresholds describe each transition separately, not a cumulative XP total. Changing thresholds does not erase progress already stored in a save. Existing save/grade reconciliation is implemented in `restore_progress`.

## 4. Damage and XP contracts

Public/configured HP is scaled by 100 relative to native health records: `HP_SCALE = 100.0`. Keep native records normalized. Multiplying every stored enemy health value by 100 would also change native phase thresholds, regeneration and death logic.

For example, a configured hit of 75 HP is 0.75 native health. At `xp_per_damage = 0.12`, an accepted 75 HP loss gives 9 XP. If only 50 HP remain, the maximum credit is 6 XP. Queueing a hit, touching an enemy or drawing an explosion is not itself evidence of HP loss.

### Follow a projectile hit

1. A hero/projectile module creates a native `HitTemplate` and delivers it through the line, sphere or direct hit services.
2. `Events::queue` recognizes an eligible target and weapon, allocates or reuses an attack instance and queues a `HitRecord` with immutable `Attack` attribution. Missile/grenade direct sweeps can be contact-only; the explosion owns their damage.
3. The scheduler publishes queued records and the enemy consumes a hit through `World::get_hit`. Native phase/state/weapon restrictions still decide acceptance.
4. `damage::resolve` deduplicates the independent event and restores its configured damage across native damage cooldowns. Native reaction/stagger state remains distinct from HP loss.
5. A matching accepted health write calls `award_health_write`, which caps XP to actual loss and routes it to the immutable weapon ID. Confirmed destruction-only handlers use `award_terminal` instead.
6. Source/target deletion and slot reuse invalidate live pointers and stale target state. The queued hit's `Origin` keeps its impact-time class and position.

Umbris is the regression example: a Blaster bullet disappears at impact before the boss reads it. `HitRecord::attacker_class` must still identify that hit, while the boss's phase shimmer continues to reject it during protected phases. Restoring an invalid live pointer or skipping the entire phase gate is not an equivalent fix.

### Streams, stuns and special XP

Pyrocitor and Tesla use configured HP/s, not damage per rendered particle. Damage accumulates per source, target and weapon over actual contact ticks. `damage_interval_ticks = 6` at 120 HP/s records 12 HP after six contact ticks. The simulation runs at 60 ticks/s. An out-of-range gap does not contribute damage; unfinished contact budgets can resume on the same target and reset when the relevant state is removed/reset. Ammo intervals are independent of damage intervals.

A real stun pauses AI without granting damage immunity. Review `frozen_damage`, `quiet_damage` and the scheduler when changing this behavior. Preserve native knockback, hurt animation timing and encounter shields while changing independent HP settlement.

Attraction, successful swallowing and morphing have separate confirmed-event XP routes (`enqueue_special`, `award_special_once`, `special_health`). Doom bot contact triggers its explosion; the explosion owns damage and XP. Do not award weapon XP merely because its collision query returned a target.

## 5. Recipe: add a per-grade weapon parameter

This is an implementation recipe, not a currently supported INI field. Suppose you want `impact_size_multiplier` for Blaster visual sparks, with neutral default 1 and accepted range 0.1..8.

1. Find the actual effect consumer. For Blaster, begin at `blaster_shot.rs::impact_sparks_colored` and inspect all callers so both muzzle-obstructed and flying-shot impacts behave consistently. Change visual size only; damage and radius are separate concerns.
2. Add the key to the item-15 branch of `supported_extra_stat`. Add explicit finite/range validation in `Evolution::parse`; its generic 0..9999 numeric fallback is not the intended limit for this example. Keep values independent across all five `ExtraTrack.params` entries.
3. Read the active value at the effect consumer, with a neutral fallback:

   ```rust
   let size = crate::weapon_evo::extra(15, "impact_size_multiplier", 1.0);
   ```

   Integrate that value into the existing spark size calculation. This line alone does not implement the feature. Decide whether an in-flight projectile should capture its grade's value at spawn, as its shot colors do, or read the current active grade at impact; document and test the chosen behavior.

4. Add the key to `schema.EXTRA[15]`, plus an English `LABELS` entry and `HELP` explanation. Give it a matching item/section-specific bound in `launcher_core.numeric_range`. Add it to `INTEGER`, `BOOLEANS` or `COLORS` only when its representation requires that type; this example is a float.
5. Add the neutral value to all five Blaster grade sections in the tracked defaults/example. If existing user profiles should show the control, add a missing-only insertion to the normalization path in `migrate_ini.convert`, preserving existing values. Regenerate the optional embedded profile.
6. Add parser tests for different grade values and invalid inputs, a meaningful effect-consumer regression where feasible, and launcher tests proving acceptance, bounds and preservation of edited presets. Verify that missing values preserve the chosen fallback.
7. If older runtimes cannot safely accept the new session profile, update the runtime capability and launcher requirement together, as described below. Rebuild and test both the runtime and GUI before packaging.

For Bomb Glove or Pyrocitor controls, use their typed `Stats`/`PyroStats` structures and parse branches. Copying only the `ExtraTrack` recipe would miss their publication/consumption paths.

## 6. Recipe: support another enemy safely

Find the enemy's class handler and damage record; class IDs and inventory IDs are separate namespaces. Check the native header in `creature::header`, the health offset, how damage is resolved, the exact HP write or confirmed kill branch, and state/phase restrictions. Determine whether a turret is destructible or merely retracts temporarily.

Add a class to `donor_class` only after confirming it is an intended XP donor. Add it to `terminal_class` only if its native destruction path needs a logical HP budget instead of numeric health; do not use that list to work around an unknown record layout. Wire actual HP writes through the shared helpers or use `award_terminal` at the confirmed weapon-destruction branch. Preserve non-damage cleanup/death paths that should not credit a weapon.

Test a nonlethal hit, a second independent hit during cooldown, replay of the same hit, overkill, source deletion, target slot reuse and any native phase/weapon protection. If the class relays hits through child parts, check event identity and requeueing: one projectile touching several parts must not silently multiply its damage to the same logical enemy.

## 7. Save compatibility

`weapon_evo_save.rs` defines 15 stable inventory IDs and a versioned binary companion format. Records contain grade and fractional XP. A native-save digest binds the companion to its save, and its own checksum validates the companion. Preparation occurs before the native save commit, so a failed commit cannot mix slots.

Preserve the ID order, `MAGIC`, encoded layout and checksum rules for compatible changes. Adding a weapon or altering the on-disk layout requires an explicit format/migration design, not just extending an array. Ordinary balance changes should not rewrite native save bytes.

`sync_progress` gives newly loaded/new-game state precedence over stale live-world progress. Combat event state is also tied to a combat epoch. Test load, new game in a different slot, repeated damage afterward and interrupted saves when changing progression or lifecycle code. Do not reset the XP track on every world tick to fix a single stale-event problem.

## 8. Locked movement, animation and camera

`combat_strafe` separates body facing from resolved world movement. The release path uses actual displacement (`hero.eff`) so backward motion stays backward and walls do not inject momentum. Change `configure`, `separate_motion` and `release_facing` together when altering that transition. `gameplay_pad_for_weapon` preserves the physical pad for menus and maps R2 to weapon fire, or wrench attack, only while locked.

`combat_legs` layers leg pose/clock changes over the native upper body. Backward-sector thresholds currently use user coordinates right=0, forward=90, left=180, with the reverse sector beginning at 200 and ending before 340 degrees. The current code uses 7 ticks into idle and 8 ticks out for direction reversal; use the code constants as authority rather than earlier conversation timings. Clean running modifiers and stale blend state are part of the implementation, not cosmetic optional cleanup.

Camera changes belong in `follow_camera.rs` as well as input integration. Test manual pitch above/below center, option-controlled speed, release easing, level-camera triggers and transitions into shops/cutscenes. For HUD changes, test both equipped weapon display and the shop's selected ammunition entry; they intentionally need not identify the same weapon.

## 9. Test and debug workflow

Run from the root. These test filters select existing regression groups without extracted game data:

```sh
python -m unittest discover -s launcher/tests -v
cargo test --release --locked --no-default-features -p rc-game --lib real_hp
cargo test --release --locked --no-default-features -p rc-game --lib umbris_beast
cargo test --release --locked --no-default-features -p rc-game --lib weapon_evo
```

The focused groups overlap. After relevant changes, run the full `rc-game --lib` test command and engine build from the README. The optional embedded profile can be tested with `cargo test --release --locked -p rc-game --lib --features weapon-evo-release fixed_release_tests` when maintaining that build mode. Game-data-dependent integration tests are documented separately in the upstream workflows.

Useful test locations are `moby_update/real_hp.rs`, `moby_update/creature/tests.rs`, Umbris `hit_origin_tests`, `weapon_evo.rs`, `weapon_evo_save.rs`, and `launcher/tests/`. Source-build tests mock Cargo to check ordering and failure handling; they are not evidence of Rust compilation.

| Symptom | First checks |
|---|---|
| A new key is rejected | Rust parser allow-list and branch, Python `allowed_keys`, type and bounds |
| A key is accepted but does nothing | Active grade, publication to `LEVEL_VALUES`, actual consumer and runtime path |
| A new control is absent in an old preset | GUI renders document keys; check missing-only migration |
| Hits work only near the muzzle | Deleted/reused source pointer, immutable origin, body/child-part acceptance, configured reach |
| Damage changes but XP does not | Matching health record/header and health write; immutable weapon attribution; confirmed terminal path |
| XP repeats while an explosion lingers | Instance allocation, per-target deduplication and terminal settlement |
| A reticle locks beyond projectile reach | Target search and effective projectile lifetime/speed/range calculation |
| A preset appears ignored | Session INI path/log, selected executable, private real-HP marker and last-preset selection |
| Progress stops after new game/load | `sync_progress`, restored state, combat epoch and lifecycle reset |
| Shop ammo shows the wrong weapon | Selected vendor entry versus equipped item in HUD state |

Launcher sessions are stored in ignored `launcher/runs/`: inspect `session.json`, `weapons.ini` and `game.log` to see the exact input and executable used. Developer/debug variables such as `RC_DEBUG_HIT` are deliberately stripped from normal player launches by `game_environment`; do not assume setting them in the parent shell enables them. Use a controlled development launch or deliberately adjust the developer path when debugging.

## 10. Versions and release boundaries

| Identifier | Purpose | Current location |
|---|---|---|
| ReRAC workspace version | Base runtime/crate version | Root `Cargo.toml`, currently 0.3.4 |
| Compatible ReRAC runtime | Runtime release targeted by Weapon Evolution | 0.3.4-alpha, using the modded game executable supplied with the mod or built from this checkout |
| Weapon Evolution version/tag | Mod source/release identity | Repository docs and Git tags, patch 0.1.6.50 / public release 0.3.2 |
| Launcher version | Frozen launcher identity and cache/self-test | `launcher_core.VERSION`, currently 0.10.0 |
| Runtime configuration capability | Reject unsupported INI/runtime combinations | `disc_source::version_json`, currently 48 |
| Source helper runtime requirement | Ensure staged game is new enough | `editabile.editable_version`, currently minimum 48 |
| GUI runtime requirement | Profile-format compatibility | `launcher_core.required_config`, profile-dependent, up to 48 |

Patch identity and configuration capability have different purposes. Patch 49 changes lifecycle behavior without adding INI fields, so capability remains 48 and launcher remains 0.10.0. The source build helper requires capability 48; the GUI negotiates the fields in the chosen profile. Always inspect the exact patch identity when verifying a bug fix.

Some inherited module comments retain historical patch labels. Use the actual capability checks, source provenance and tests to identify the current contract. Fixed-profile runtimes are intentionally rejected by the editable GUI workflow; maintain/test that optional build mode separately.

Build the game first. A launcher EXE build can reuse a cached GUI binary and cannot compile a gameplay fix into an old `runtime/rerac.exe`. Validate `--version-json`, stage the fresh runtime, then run `BUILD_FINAL_WINDOWS_EXE.bat` and test the extracted player ZIP. Keep the runtime shaders and collected licenses with their binaries. Do not include extracted disc data, saves, local presets or machine-specific settings in the source repository.

## 11. Maintain defaults and the source map

The source map hashes record exported source content. They intentionally detect later edits; an indexed-file change requires refreshing that record before the export checker will pass. After reviewing intentional changes, run the following from the repository root to refresh existing entries and regenerate the optional embedded profile:

```python
from pathlib import Path
import hashlib
import json
import sys

root = Path.cwd()
sys.path.insert(0, str(root / "launcher"))
from migrate_ini import runtime_profile

default = (root / "launcher/weapons.default.ini").read_bytes()
example = (root / "launcher/weapons.example.ini").read_bytes()
assert default == example, "Review the public default/example difference first"
(root / "crates/rc-game/src/weapon_evo_release.ini").write_bytes(runtime_profile(default))
index = root / "mod/source-map.json"
record = json.loads(index.read_text(encoding="utf-8"))
for item in record["files"]:
    item["sha256"] = hashlib.sha256((root / item["path"]).read_bytes()).hexdigest()
index.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
```

Review new/deleted integration files separately: this recipe updates existing entries only. Update `MOD_FILES.md`, configuration reference and release provenance when the scope changes. The included delta is specifically the historical export delta from the supplied snapshot; it is not automatically regenerated by the hash recipe and is not an official upstream commit history.

Stage the reviewed source/documentation changes before running `python tools/mod/check_source.py`, because that check examines tracked files. Its capability check currently expects 48 and must be deliberately revised if the exported runtime capability changes. Preserve the ISC license and ReRAC attribution. The original source export's Git tag stays attached to its original commit; make new commits/tags for later changes rather than rewriting published history.


## 12. Current experiments and respawn ownership

Read [CURRENT_IMPLEMENTATION.md](CURRENT_IMPLEMENTATION.md) for the per-weapon implementation map and [RESPAWN_LIFECYCLE.md](RESPAWN_LIFECYCLE.md) before modifying object-table replacement. Current regression tests and generated CONFIG_REFERENCE.md take precedence over historical patch notes.

Refresh reviewed source hashes with `python tools/mod/refresh_source_map.py`; pass `--add path/to/new_module.rs` to index a new integration file. Stage new source files before running the export checker.

Source-map hashes use Git LF normalization for BAT files so Windows CRLF checkout does not change integrity results. Other source bytes and binary icons are hashed exactly.

## v51 ammunition-based arsenal

Start with [AMMO_UNLOCK_MODE.md](AMMO_UNLOCK_MODE.md) for ownership reconciliation, per-slot state, independent pack drops, shop gates and the ordinary R.Y.N.O. ammo fix. Native save chunks are unchanged; the mod companion has backward-compatible v1 reading and writes version 2. Keep mode changes independent of the XP synchronization/combat epoch.
