# ReRAC Weapon Evolution

Public release **0.3.2**: gameplay patch **0.1.6.50**, configuration capability **48**, English preset launcher **0.10.0**, built on the supplied ReRAC **0.3.4** source snapshot.

**Runtime compatibility: ReRAC 0.3.4-alpha**, using the modified game runtime supplied with Weapon Evolution or built from this repository.

**Authorship:** TomRus47 is the author of the Weapon Evolution mod only and was not involved in developing ReRAC or the original Ratchet & Clank game. All credit for the underlying port and original game belongs to their respective developers.

This is a source repository. It contains the modified Rust workspace, launcher source, approved default weapon configuration, tests and Windows build tools. The game data, compiled game, compiled launcher, saves and local settings are excluded.

The full ReRAC source base is included because the mod integrates with many gameplay and rendering systems. No earlier patch installation is required. [MOD_FILES.md](MOD_FILES.md) identifies the mod modules and integration points; [mod/source-map.json](mod/source-map.json) provides their hashes.

See the [release notes](docs/releases/v0.3.2.md), [current implementation map](docs/mod/CURRENT_IMPLEMENTATION.md), and [respawn investigation](docs/mod/RESPAWN_LIFECYCLE.md) for current behavior. See the [loading and shotgun investigation](docs/mod/LOADING_AND_SHOTGUN.md) for the v50 fixes and measurement limits. The v40 audio investigation is historical; temporary probes have been removed.

## Features

- Independent weapon XP and five configurable grades; localized in-game weapon names and colored grade icons.
- Real HP damage, actual-health-loss XP attribution and independent projectile/explosion hit tracking.
- Per-grade ammunition, damage, firing intervals, stream intervals, targeting ranges, gold behavior and weapon effects.
- Camera-locked movement and combat, directional leg animation and R2 firing while locked.
- Weapon HUD name, grade, XP bar and synchronized fades.
- English launcher with optional saved presets, editable defaults, portable runtime selection and a standalone Windows EXE build.
- Preserve encounter phase protection while fixing independent weapon hit handling.
- Experimental cluster bombs, triple flames, crate ammunition, a second Tesla chain, continuous-missile or 35-pellet R.Y.N.O., flame mines and a Drone Device Sand Mouse companion.
- XP below the pause-menu weapon-name panel, up to ten ordered drones, directional boot poses and expanded/stabilized locked-camera aim.
- Death-reload cleanup for stale effects, companion IDs and reaction state; uninterrupted accepted stream damage with separate reaction cooldowns.

## For developers modifying the mod

Start with the [Developer guide](docs/mod/DEVELOPER_GUIDE.md). It maps concrete tasks to files and symbols, follows the damage/XP and save lifecycles, and gives implementation recipes for new INI controls and enemy support. It also explains camera/animation integration, runtime compatibility, debugging and release checks. The [Configuration reference](docs/mod/CONFIG_REFERENCE.md) documents the public fields by weapon with launcher types, bounds and help text.

## Build on Windows x64

Build the modded **ReRAC 0.3.4-alpha** runtime from this checkout before packaging the launcher.

Authoring requires Rust 1.99.1 with the MSVC toolchain, Visual Studio C++ build tools and a Windows SDK, plus 64-bit Python 3.10 or later with Tcl/Tk. The game uses the dependencies pinned in `Cargo.lock`. The launcher build installs its pinned PyInstaller version in a local tools environment.

1. Extract or clone this repository into any writable folder.
2. Open **x64 Native Tools Command Prompt for VS**, change to that folder and run:

   ```bat
   01_BUILD_GAME.bat
   ```

   This runs the `rc-game` library tests, builds the editable game and stages the runtime in `launcher/runtime/`. It builds directly from this checkout; it does not patch a separate ReRAC installation or require game data to compile.

3. Open the editor:

   ```bat
   02_OPEN_LAUNCHER.bat
   ```

   The launcher creates `launcher/weapons.ini` from the tracked example on first use. Select your extracted game data if automatic discovery does not find it. The directory must contain `toc.bin` and `extract-info.json`. Existing ReRAC data under `%LOCALAPPDATA%\rerac\games\rac1\data` can be detected automatically.

4. To generate the standalone player package after building the game:

   ```bat
   BUILD_FINAL_WINDOWS_EXE.bat
   ```

   Output is written under `launcher/dist/`. Share the complete generated player ZIP. It includes the launcher EXE, patched game runtime, shaders, configuration and collected launcher licenses. Players do not need Python, Rust or earlier patches.

The tracked `launcher/launcher.ico` is the supplied custom icon. The build cache tracks changes to this file.

## Configuration

The tracked starting profile is `launcher/weapons.default.ini`; `launcher/weapons.example.ini` contains the same settings. Personal `launcher/weapons.ini`, presets, runtime, build output and launcher settings are ignored by Git.

With no preset selected, the launcher reads the current `weapons.ini`; creating a named preset is optional. It remembers an explicitly selected preset across sessions. Each launch creates a private session profile and injects the real-HP marker needed by the runtime. Use the launcher or the supplied helper to activate the complete mod environment.

The normal build uses `--no-default-features`, with editable external configuration. The optional `weapon-evo-release` feature embeds a profile; its checked-in profile is synchronized with the approved English defaults. Changing the external default does not automatically regenerate the optional embedded copy.

## Tests

Run from the repository root:

```sh
python tools/mod/check_source.py
python -m unittest discover -s launcher/tests -v
cargo test --release --locked --no-default-features -p rc-game --lib
cargo build --release --locked --no-default-features -p rc-engine --bin rerac
```

The Windows build helper runs the last two commands with an explicit checkout-local target directory. Repository export checks and launcher tests can run without game data. Some upstream integration tests require an extracted game; the focused commands above do not run those integration suites.

See [docs/mod/VERIFICATION.md](docs/mod/VERIFICATION.md) for the checks actually performed on this export. A successful source export does not substitute for a Windows build and gameplay test.

## Source layout

| Path | Contents |
|---|---|
| `crates/rc-game/` | Weapon evolution, save state, combat, enemy hit handling and gameplay integration |
| `crates/rc-engine/` | Runtime, HUD, camera/input integration and shaders |
| `launcher/` | Python launcher, default INI, tests and Windows EXE/package builder |
| `mod/` | Source map, provenance and delta from the supplied starting snapshot |
| `tools/mod/` | Source export verification |
| `docs/mod/` | Mod architecture, build/export notes and GitHub publishing instructions |
| Other `crates/`, `tools/`, `docs/` | ReRAC base source and developer documentation |

## Provenance and license

ReRAC and its contributors retain their existing [ISC license](LICENSE) and copyright notice. The historical ReRAC README is preserved as [README.ReRAC.md](README.ReRAC.md). This mod is an unofficial derivative and is not affiliated with Sony Interactive Entertainment or Insomniac Games.

The supplied base archive already contained an earlier Weapon Evolution version. Its original upstream Git commit was not supplied, so this repository does not claim to reproduce official upstream Git history. The included patch compares against that exact supplied snapshot. See [mod/provenance.json](mod/provenance.json).

For player ZIPs, draft releases and the Windows build workflow, read [docs/mod/PUBLISHING.md](docs/mod/PUBLISHING.md).

To package an already edited local INI without installing the tracked defaults, use:

```bat
py -3 launcher\package_launcher.py --exe launcher\runtime\rerac.exe --ini launcher\weapons.ini --no-presets
```

The Windows release workflow builds and smoke-tests both executables, packages the tracked default, uploads checksums and prepares a **draft** release. Gameplay validation is still required before publishing.
