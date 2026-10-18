# Mod architecture

For step-by-step changes and code navigation, read the [Developer guide](DEVELOPER_GUIDE.md) and [Configuration reference](CONFIG_REFERENCE.md).

## Runtime configuration and saves

`crates/rc-game/src/weapon_evo.rs` owns the INI parser, per-weapon progression, grade parameters, published weapon settings and XP calculation. `weapon_evo_save.rs` exchanges progression with the game save system. The launcher edits the public INI and creates a session copy with `damage_mode = real_hp` before starting the editable runtime.

## Damage and XP

`moby_update/real_hp.rs` tracks independent attacks, streams and per-target settlement. Public HP uses 100 units for one native health unit; native AI health records remain normalized so encounter thresholds and phase behavior retain their original scale.

Each independent attack carries a stable weapon ID, instance ID, configured damage and impact-time source class/position. Collision delivery queues the hit; accepted native health writes award XP for actual health loss. Deleting a projectile clears dangling source pointers without discarding its already confirmed hit. Boss phase guards and other class-specific acceptance rules still run.

`moby_update/services.rs`, the shared creature damage/reaction layer and enemy class handlers connect these events to the port. Projectile, explosion and continuous-flow weapon modules provide the relevant attack templates. The source map lists the integration files.

## Movement, camera and HUD

`hero/combat_strafe.rs` and `hero/combat_legs.rs` handle locked combat movement and the split directional animation. Camera and input integration are in the gameplay/engine modules. HUD rendering in `rc-engine` applies the weapon name, grade, background and XP display with ammunition HUD timing.

## Launcher and build tools

`launcher.py`, `launcher_core.py`, `schema.py` and `migrate_ini.py` implement the editor and session profiles. `frozen_support.py` manages frozen application paths and native process startup. `build_launcher_exe.py` builds and checks the Windows launcher; `package_launcher.py` packages the already compiled runtime; `final_release.py` installs the approved default once and invokes packaging.

`launcher/editabile.py` has been adapted for this source repository: it discovers the checkout relative to itself, tests/builds the existing Rust sources and stages the resulting runtime. Historical source patch installers and source backups are intentionally excluded.
