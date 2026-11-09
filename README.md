# ReRAC Weapon Evolution 0.5.1

Independent mod for **ReRAC 0.3.8-alpha**. This repository contains the complete
Rust workspace, modified runtime integrations, standalone launcher, approved
configuration, build tools, tests and implementation documentation.

**To play, download the Windows x64 ZIP from the [0.5.1 release](https://github.com/TomRus47/rerac-weapon-evolution/releases/tag/v0.5).**
Extract everything and open `ReRAC_Weapon_Evolution.exe`. Use its included
modified `runtime/rerac.exe`; select your extracted Ratchet & Clank data if
needed. The data folder must contain `toc.bin` and `extract-info.json`.
Players do not need Python, Rust or older patches. Game data is not distributed.

GitHub's **Source code** archives are for reading, modifying or building the
project, not for starting the game. The release has no extra JSON/checksum or
duplicate source attachments. See [0.5.1 fixes and source notes](docs/releases/v0.5.1.md)
and the [0.5 release notes and update instructions](docs/releases/v0.5.md).

## Fixes in 0.5.1

- Restore native explosion damage and weapon XP attribution for Bomb Glove,
  Mine Glove, guided Visibomb and exploding chickens. The experimental
  Trespasser now uses separate, ownership-checked attack metadata.
- Return the name/XP panel to the recalled weapon after wrench XP feedback
  when Circle/R2 or another equipment path completes a hand-item change.

The current `main` source includes both fixes through runtime **0.1.6.142**.
The existing release is named 0.5.1 but still uses the historical `v0.5` tag;
its automatic Source code archives remain tied to that tag. Download the
[current main source](https://github.com/TomRus47/rerac-weapon-evolution/archive/refs/heads/main.zip)
to obtain these fixes. This source synchronization does not retag the release
or replace its player download.

## Features

- Independent weapon XP and eight configurable grades, with native localized
  names, bright grade icons, Gold textures and NG+ Gold purchase progression.
- Real-HP damage, accepted-loss XP, per-instance projectile/explosion hits and
  independent continuous-damage, ammunition and reaction clocks.
- Editable weapon stats, colors, ammo capacities and optional experiments,
  including lava streams, flame mines, cluster bombs, crate ammunition,
  electrical rays, R.Y.N.O. shotgun/continuous missiles and companions.
- Locked-camera combat, directional leg animation, R2/Circle activation, saved
  D-pad gadget shortcuts, two Quick Select rings, optional world pause and
  per-grade contextual R1 experimental modes.
- Configurable encounters, enemy HP/movement/attacks, extra enemies, NG+ scaling,
  bolt/XP chains and XP-driven Nanotech progression with compact layered HUD.
- Ammo-Survival, weapons-only discoveries and a mixed Randomizer All mode with
  saved layouts, route constraints, optional extra sites and grey Bolt Packs.
- Optional Swingshot grappling/enemy pull, contact-based Trespasser capture,
  Hydrodisplacer Freezer and Hologuise speed settings, preserving native puzzle
  interactions. See [gadget implementation and current release handoff](docs/mod/LOCAL_TEST_V140.md).
- Five-language standalone configuration editor (English, Italian, French,
  German, Spanish), lossless profiles, optional presets and supported live tuning.

## Randomizer testing

**Randomizer All is still being tested and may generate an incompletable seed.**
Code-level route checks do not establish every physical path or mission script.
If blocked, save and close the game, then ZIP the complete randomizer memory card,
including native saves and matching `.wev` companions. The default Windows path:

```text
%APPDATA%\rerac\memcard\randomizer\
```

Use the startup log's actual memory-card path if a custom save root is configured.
Keep the region-specific folder and companions together. Include your mod/runtime
version, planet, equipment and routes checked in a repository issue. Existing
saves keep their saved seed, placements, receipts and location selection.

## Build and modify

Start with the [developer guide](docs/mod/DEVELOPER_GUIDE.md),
[configuration reference](docs/mod/CONFIG_REFERENCE.md),
[implementation map](docs/mod/CURRENT_IMPLEMENTATION.md) and
[source map](MOD_FILES.md). Upstream integration boundaries are recorded in
[ReRAC 0.3.8 notes](docs/mod/RUNTIME_038.md). Historical `LOCAL_TEST_V*.md`
documents retain their original scope; the current source and release notes
identify the shipped behavior.

Windows authoring requires Rust MSVC, Visual Studio C++ x64 tools/Windows SDK and
64-bit Python 3.10+ with Tcl/Tk. Dependencies are pinned in Cargo.lock; the
validated toolchain is Rust 1.99.1. From an x64 Native Tools prompt:

```bat
01_BUILD_GAME.bat
02_OPEN_LAUNCHER.bat
```

The build stages this checkout's editable runtime in `launcher/runtime`.
The editor initializes a missing personal `weapons.ini` from the public example;
existing profiles are not overwritten. `BUILD_FINAL_WINDOWS_EXE.bat` builds a
local player ZIP after the runtime. See [release workflow](docs/mod/RELEASE_05.md)
for exact Windows/source identity and publication checks.

The 0.5.1 gameplay snapshot is **0.1.6.142**, capability **97**, with launcher
**0.30.2**. Component versions are independent from the public mod version.
The supplied v142 kit records 1,528 host gameplay tests passing with three
unchanged heavy seed surveys filtered. See the
[source synchronization checks](docs/mod/SOURCE_SYNC_051.md) for checks rerun on
this integration and their scope. Windows runtime/GUI builds remain separate.
See [verification details and limitations](docs/mod/VERIFICATION.md).

## Authorship and rights

Weapon Evolution is an unofficial, independent mod by **TomRus47**, who develops
this mod only and is **not associated with the development of ReRAC or the
original Ratchet & Clank games**. No rights to the original games, characters,
trademarks or game assets are claimed. They belong to their respective owners.
Huge thanks to the ReRAC developers and contributors: their work on the port and
continuing improvements make mods like this possible. Upstream licenses and
contributor attribution remain applicable. See [credits](docs/mod/CREDITS.md)
and [LICENSE](LICENSE). This repository contains no original disc data, saves,
compiled game executables or personal launcher settings.
