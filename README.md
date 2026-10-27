# ReRAC Weapon Evolution

Weapon Evolution **0.3**, current **development snapshot**: gameplay **0.1.6.106**, configuration capability **89**, launcher **0.28.9**, based on **ReRAC 0.3.6-alpha**. This main-branch update is not a new release. The previously published Windows/source assets and their release tag remain unchanged.

**Authorship and rights:** Weapon Evolution is an unofficial, independent mod by TomRus47. TomRus47 develops this mod only, not ReRAC or the original Ratchet & Clank games, owns no rights to their games, characters, trademarks or game assets, and is not affiliated with their rights holders. Upstream licenses and contributor credits remain applicable. See [credits](docs/mod/CREDITS.md) and [LICENSE](LICENSE).

This repository includes the complete supplied Rust workspace, mod integrations, launcher source, approved default profile, tests and build tools. It does not include compiled executables, game data, saves or personal settings. It can be built directly without installing previous mod patches. Upstream source is included to make the modified runtime reproducible, not to claim authorship of the port.

See the [v95 route audit](docs/mod/RANDOMIZER_ROUTE_AUDIT.md) for corrected Blarg/Orxon access, conservative Gold Bolt gates and the saved-layout inspector. These are modeled access checks; physical gameplay verification remains necessary.

See [v106 wrench blast geometry](docs/mod/SLAM_VISUAL_V106.md) for the current correction and [v102 combat feedback and HUD](docs/mod/FEEDBACK_HUD_V102.md) for the new controls, local test/update workflow and verification boundaries.

## Current development features

- Accepted-hit red feedback and finite repeating body-fire cycles independent of stun; per-grade wrench visual ignition and an optional independent ground-slam explosion.
- Swimming-safe weapon HUD, per-grade Triangle-only wrench HUD, alternative weapon-bar positioning and model-free randomizer rewards represented by a Gadgetron ammo crate.
- Independent weapon XP, eight configurable grades, localized game names and bright grade icons. Weapons reach V5 through XP, require their NG+ Gold purchase for V6, then progress to V8. The wrench uses XP for its upper grades in NG+.
- Real HP, per-attack projectile/explosion identity, actual accepted health-loss XP, independent damage/ammo clocks and separate enemy reaction cooldowns.
- Per-grade ammo, damage, targeting, ranges, colors, Gold appearance and optional weapon replacements/enhancements, including lava streams, flame mines, grenade Visibomb and RYNO shotgun.
- Enemy health, population, movement, perception, pursuit and projectile tuning, with optional health-class damage to Ratchet and XP-driven Nanotech progression.
- Seeded weapon discoveries at registered collectible locations, isolated randomizer save state, access constraints and underwater collection; optional Ammo-Survival Mode.
- NG+ Gold shop progression, shared bolt/weapon-XP/Nanotech-XP reward chain and separate NG+ encounter scaling.
- Locked-camera movement and directional legs; R2/Circle firing during ordinary or locked movement; **L1 uses first-person aiming and R1 crouches** in direct Ratchet gameplay.
- Optional **D-pad Quick Select** for weapons and all equippable gadgets. Hold L1+direction in paused Weapons/Gadgets to bind the highlighted owned item. Colored borders identify assignments; the help caption appears only in Weapons. Repeating a boots/helmet shortcut removes that equipped item. Repeating a Clank-pack shortcut cycles owned Heli-Pack → Thruster-Pack → Hydro-Pack after its first selection.
- Optional **R1_Experimental**: hold Triangle and cycle Pyrocitor normal/lava, Mine Glove normal/fire, Visibomb guided/grenade, or RYNO native/continuous/shotgun. Other enhancements remain launcher-only. Modes and bindings are saved per slot; confirmations use normal weapon-menu audio.
- Synchronized weapon HUD, pause-menu weapon XP and a separate Nanotech XP HUD bar. Multilingual launcher (English, Italian, French, German, Spanish), Off/On selectors, optional named presets and a standalone Windows EXE builder.

The arena experiment was removed; it is not part of this snapshot. Temporary audio diagnostics are not enabled.

See [current implementation](docs/mod/CURRENT_IMPLEMENTATION.md), [saved controls](docs/mod/SAVED_CONTROLS.md), [configuration reference](docs/mod/CONFIG_REFERENCE.md) and [source map](MOD_FILES.md). Earlier versioned notes remain historical; current source/configuration documentation takes precedence.

## Build and test on Windows x64

Authoring requires the Rust MSVC toolchain, Visual Studio C++ x64 tools and a Windows SDK, plus 64-bit Python 3.10+ with Tcl/Tk. Dependencies are pinned in `Cargo.lock`; Rust 1.99.1 is the validation toolchain. Build from an x64 Native Tools command prompt:

```bat
01_BUILD_GAME.bat
02_OPEN_LAUNCHER.bat
```

The first command tests rc-game, builds this checkout's editable ReRAC 0.3.6-alpha runtime and stages it in `launcher/runtime/`. The editor bootstraps a missing personal `launcher/weapons.ini` from the tracked example; existing personal values and presets remain local. Select extracted game data containing `toc.bin` and `extract-info.json`. Existing ReRAC data under `%LOCALAPPDATA%\rerac\games\rac1\data` can be detected automatically.

To build a local standalone player ZIP after compiling the game:

```bat
BUILD_FINAL_WINDOWS_EXE.bat
```

This creates a local package only; it does not publish a GitHub release. It includes the patched runtime, launcher, shaders, configuration and licenses. Players extract the entire package and open `ReRAC_Weapon_Evolution.exe` directly; they do not need Python, Rust or earlier patches. Do not import a standalone player ZIP into the official ReRAC launcher. Game data is supplied separately by the player.

The tracked custom `launcher/launcher.ico` is included in the EXE build fingerprint. See [build layout](docs/mod/BUILD_LAYOUT.md) for source/launcher/player folder distinctions.

## Configuration and saves

`launcher/weapons.default.ini` and `launcher/weapons.example.ini` are identical public defaults. Personal `weapons.ini`, presets, runtime, logs and launcher settings are ignored by Git. With no preset selected the launcher uses the current `weapons.ini`; named presets are optional and an explicitly selected preset is remembered. The internal real-HP marker is injected only into private runtime profiles.

The optional embedded-profile feature uses the same reviewed default, with its internal session marker. Normal builds remain editable with `--no-default-features`.

Weapon/health/randomizer progression and shortcut bindings/modes are stored in the existing content-addressed `.wev` save companions. Format v8 reads previous formats 1–7 without replacing their progression. Normal equipped items remain in native saves. See [saved controls](docs/mod/SAVED_CONTROLS.md) for slot isolation, repeat actions and input restrictions.

## Verification and maintenance

```sh
python tools/mod/check_source.py
python -m unittest discover -s launcher/tests -v
cargo test --release --locked --no-default-features -p rc-game --lib
cargo test --release --locked --no-default-features -p rc-game --test controls_api --test golden_render_api --test encounter_effects_api
cargo build --release --locked --no-default-features -p rc-engine --bin rerac
```

See [verification](docs/mod/VERIFICATION.md) for completed checks and remaining Windows/gameplay validation. Start extensions with the [developer guide](docs/mod/DEVELOPER_GUIDE.md), [configuration reference](docs/mod/CONFIG_REFERENCE.md) and [module index](MOD_FILES.md). Preserve damage/XP attribution, native phase shields, loader ownership and per-save boundaries when rebasing onto newer runtimes.

`mod/development.json` describes this main-branch snapshot. `mod/release.json` still describes the previous published release; it has deliberately not been advanced. Release packaging/publication scripts reject a runtime that does not match an explicitly prepared release descriptor. Source pushes never publish releases; manual release preparation and authorization are separate.

## Source provenance

The supplied source archives already contain earlier mod work. No exact official upstream commit was supplied, so this repository does not claim to reproduce upstream Git history. [Provenance](mod/provenance.json) records the supplied baseline and preserves historical export information. The historical port README is retained as [README.ReRAC.md](README.ReRAC.md); the current complete workspace uses ReRAC 0.3.6-alpha, including its separate rc-game2 module.
