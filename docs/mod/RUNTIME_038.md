# ReRAC 0.3.8-alpha integration — local v112 test

Weapon Evolution **0.1.6.112**, configuration capability **90**, compatible with
launcher **0.29.3**. This source kit includes the supplied complete v111 fixes
and the official **0.3.7-alpha → 0.3.8-alpha** delta. No GitHub repository,
release, draft or tag is changed by this local kit.

Weapon Evolution is an independent mod by TomRus47. The author develops this mod
only, is not associated with ReRAC development and owns no original-game rights
or assets. Thanks to the ReRAC developers for their continuing work and for the
runtime that enables this project. See [credits](CREDITS.md) and upstream licenses.

## Source boundaries

- Input: the author's supplied v111 local source kit, whose 1,378 source files
  matched its inventory before modification. Compiled/player files and local
  paths from that upload are not imported into this source export.
- Official 0.3.7 base: `f2dfbb84d7025d475ce8e39f9ab4ae345ab35133`.
- Official 0.3.8 head: `46c1948b604445cbbbcbdfaee4cb15bb7cc899ac`.
- All **53 changed paths / 25 commits** are included. Every upstream base/head
  file was verified by its Git blob hash before a three-way merge with v111.
- `mod/upstream_038.json` records exact upstream identities, supplied v111 hashes,
  merged hashes and integration decisions. Older inventories are historical.

The official release is https://github.com/re-rac/rerac/releases/tag/v0.3.8-alpha.

## Official changes included

- Restore the ship when returning to Gemlik after its escape; distinguish that
  later visit from death reloading the current adopted ship.
- Apply PS2-style warm gold lighting and metal blending to weapons in hand,
  previews and acquisition demos.
- Add Decoy Glove and Glove of Doom landing previews. Mines, decoys and Doom
  canisters inherit carrier momentum and ride the Metropolis train/platforms.
- Restore Kalebo III's placed breakroom chickens' animation, sound and hit intake.
- Align first-person wrench throws with the rolled camera on magnetic surfaces.
- Persist the chosen game language and choose a supported OS language on first
  start. This is separate from the mod launcher's language selector.
- Add the Fill aspect ratio for ultrawide and other window shapes.
- Sample all connected controllers and install the upstream Linux 8BitDo mapping.
- Support `memcard_dir` in port settings and an existing `memcard` directory next
  to the runtime EXE, while retaining the environment override and default path.

## Merge decisions

- **Wrench:** retain v111's single-wrench entry guards, movement option, captured
  attack tuning/XP/fire and regression tests; adopt the camera-row launch frame
  and upstream magnetic/mirrored-camera test.
- **Gloves/mines:** combine carrier velocity with the mod's actual release birth
  tick, per-grade snapshots, alternate active-count/lifetime and R1 modes. Native
  seeking remains bypassed for fire mines; their own platform/light/sound cleanup
  is retained. Adapt the experimental test's release call to the new API.
- **HUD:** retain weapon/name/Nanotech bars, compositing order, reset recovery and
  the insert-batch regression, together with native language persistence/tests.
- **Gold previews:** retain the mod's gold-texture decisions and drone scaling;
  adopt the upstream warm lighting, including the custom drone preview role.
- **Startup:** initialize the SDL mapping before Bevy starts threads, alongside
  optional mod defaults and explicit local-test startup settings.
- **Saves:** select the upstream configured/environment/default root first, then
  apply the existing mod mode directory. Randomizer and weapons-only saves never
  merge with the normal card. An explicit RC_SAVE_DIR still wins; `0` still
  disables the card. No existing card is moved or rewritten by the installer.
- **Performance:** retain the mod's bounded mine explosion particle count;
  document it where the native camera-distance count appears in upstream notes.

All other v111 systems remain in place, including generic pickup/trade names,
item-specific acquisition scenes, Gold Bolt cutaway, randomized boots and zero
weapon/Nanotech XP from split amoeboid descendants. No seed or save migration is
introduced. WEVO v9 and its legacy readers remain unchanged.

## Configuration and installation

All **2,592 approved default settings / 146 sections** are unchanged. Only the
English compatibility header is advanced to 0.3.8-alpha. Default/example/embedded
profiles remain synchronized. Existing `weapons.ini`, presets and saves are never
overwritten by this updater. The existing launcher EXE and custom icon remain.

Extract the full ZIP into a new writable folder, close the game, then run
`01_COMPILE_AND_UPDATE.bat`. Select the **existing playable mod folder** with
its launcher EXE, `weapons.ini` and `runtime/rerac.exe`, not this source kit.
The script builds these bundled sources with installed Rust and Visual Studio
C++ x64 tools/SDK, verifies the runtime identity, then transactionally replaces
the runtime and matching shaders. It initializes the installed MSVC environment
and pauses at completion/error. Compilation failure leaves installed files alone;
installation failure rolls back changed files. Backups/log paths are printed.

`02_OPEN_UPDATED_LAUNCHER.bat` verifies and opens that same installation.
`03_CREATE_PLAYABLE_ZIP.bat` packages its current INI and existing launcher with
the freshly compiled runtime, without personal presets, saves or disc data.
A user-provided save directory remains configured as before. When using the
upstream portable-card feature, place `memcard` next to `runtime/rerac.exe`, not
next to the mod's GUI EXE. The selected root is printed in the runtime log.

## Verification and playtest limits

Executed results are recorded in [VERIFICATION.md](VERIFICATION.md). The local
preparation host can test Rust logic/APIs, profile preservation and updater
transactions; it cannot run the Windows/GPU game with the author's disc assets.
The native Windows build is run by the supplied installer before any replacement.

Playtest returning to Gemlik after escape and dying there; throw each glove from
the moving train; test normal and fire mines, per-grade lifetime/count/R1 options,
gold previews/demos, magnetic wrench throws, connected controllers, Fill display,
chosen language after restart, and separate normal/randomizer/weapons-only saves.
Also recheck the v111 pickup scenes, boots after travel and amoeboid XP budgets.
Randomizer seed completeness remains experimental; this runtime port makes no
new claim of universally completable seeds.
