# Layered Nanotech and local Veldin II test (v115)

Weapon Evolution is an independent mod by TomRus47, not a ReRAC development
project. The author owns no rights to the original games or their assets.
Thanks to the ReRAC developers for making this work possible. Existing licenses
and contributor credits remain applicable.

## Health and HUD

Actual health remains one integer. An accepted one-HP hit always removes one HP;
colored spheres do not implement a second damage, armor or invulnerability system.
The hard limit is **300 total usable HP**, including the original four points,
Premium (+1), Ultra (+3) and XP-earned health. `nanotech_max_health` continues to
limit the **XP bonus**, now within 0–300. Lowering it does not erase earned bonus.
The effective XP cap also accounts for native health, so a full XP bar and upgrade
notifications stop correctly at 300. Premium/Ultra purchases cannot bypass it.

The HUD displays at most 50 spheres in two centered rows. Native 4/5/8 layouts
are retained; expanded artwork and spacing keep their existing 50% scale.

| HP stored in one sphere | Color | Total full-health range |
| --- | --- | --- |
| 1 | Native blue | 1–50 |
| 2 | Purple | 51–100 |
| 3 | Red | 101–150 |
| 4 | Orange | 151–200 |
| 5 | Yellow | 201–250 |
| 6 | Green | 251–300 |

The table describes the newly added layer. At 51 max HP only the first sphere
is purple; all others are blue. At 100 all 50 are purple. HP is packed from the
first sphere toward the last, and damage empties the last occupied sphere first.
At 51 maximum and 2 current HP, the first purple sphere is the only filled one;
a one-HP hit returns it to blue, and another empties it. There is no white layer.

`hud::nanotech` owns the pure packing/color functions. The engine recolors copies
of the native orb vortex and enclosing shell textures once during HUD loading.
It preserves native alpha, artwork, highlights and draw ordering. Each draw uses
one palette variant instead of stacking a colored sphere on top. No shader
changes or additional per-sphere protection geometry are involved.

`NanotechProgress` retains its existing serialized `u32` fields. Previous saves
remain readable. A save with a bonus above 64 requires v115 or newer; retain the
previous save backup if testing an older runtime. Notifications and cosmetic
upgrade bursts use 16-bit progression counters, including upgrades above 255.
The GUI accepts the expanded bonus range and requires capability 91 for values
above 64; ordinary existing profiles keep their previous compatibility floor.

## Morph-o-Ray chain credit

A successful normal or gold conversion counts as one defeated eligible enemy for
the existing bolt/weapon/Nanotech reward-chain multiplier. It does not force the
multiplier on outside its normal NG+/randomizer rules. Credit precedes XP award,
matching numeric lethal damage. Zero XP or a disabled Morph-o-Ray XP track does
not suppress the kill count. Allocation failure gives no credit; duplicate
callbacks, dead enemies and the produced chicken cannot count again. Reusing a
dynamic enemy slot starts a new life through the existing identity reset.

## Disposable Veldin II test

After `01_COMPILE_AND_UPDATE.bat` succeeds, close the launcher/game and run
`04_TEST_NANOTECH_VELDIN_II.bat`. It directly boots level 18 with all equipment,
weapons, planet coordinates, the upgraded ship and **51/51 HP**. Final-level
encounters are active; the final battle is not pre-completed. Other planets can
be visited. Ordinary save reads/writes are disabled (`RC_SAVE_DIR=0`); user INIs,
presets and slots are untouched. Changes in this disposable run are not saved.

- **F5:** cycle maxima 50, 51, 100, 101, 150, 151, 200, 201, 250, 251, 299, 300, 8.
  Changing the maximum fully heals Ratchet for the next visual check.
- **F6:** heal to the current maximum.
- **F7:** remove exactly one HP, stopping at one. Use enemies to test death.
- Optional initial health: `04_TEST_NANOTECH_VELDIN_II.bat --hp 300`.

The test uses the installed mod runtime and its real extracted data folder,
discovered from the existing launcher settings or the standard ReRAC data path.
If neither is valid, it asks for the folder containing `toc.bin` and
`extract-info.json`. It never falls back to an absent development `extracted`
directory. Test output is written to `logs/nanotech-*/game.log` in the prepared
source kit. No audio probes are installed.

## Source overlay and installation

This small patch is applied to an extracted v113 or v114 full source kit, never
to the old checkout or playable files directly. Use v114 when already testing
v114: every unmodified file, including the Gold Bolt acquisition and space-demo
completion fixes, is copied unchanged. The overlay verifies input and payload
hashes before making an isolated source candidate. It refuses unknown changed
source files. The original source kit is never edited.

The Windows installer runs Rust tests, builds the runtime and the standalone
0.29.4 GUI, and verifies their identities before installation. The installed
launcher icon is recovered into a separate build stage. Runtime, matching
shaders and GUI are backed up/replaced in one rollback transaction. User INIs,
presets, saves and launcher settings stay outside the replacement set.

`tools/mod/create_nanotech115_kit.py` is the v115 full-source exporter for a
tracked checkout; earlier exporters remain historical tools.

`03_CREATE_PLAYABLE_ZIP.bat` packages the verified installation and its current
`weapons.ini`, without saves, personal presets or extracted game data. None of
the local scripts runs GitHub operations.
