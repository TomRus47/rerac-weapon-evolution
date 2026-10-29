# Compact Nanotech, reserved native upgrades and Veldin II test (v116)

This is a local Weapon Evolution update for ReRAC 0.3.8-alpha: gameplay
0.1.6.116, configuration capability 92, standalone launcher 0.29.5.
No GitHub repository, draft, tag or release is changed by the local kit.

Weapon Evolution is an independent unofficial mod by TomRus47. The author is not
a ReRAC developer, is not affiliated with the original game's rights holders and
owns no rights to the original games, characters, trademarks or assets. Thanks
to the ReRAC developers for their work that makes this mod possible. Existing
licenses and contributor credits remain applicable.

## Health representation

At most **30 existing Nanotech spheres** are drawn in two centered rows, at most
15 per row. Expanded spheres and their spacing retain the existing 50% scale.
The native four/five/eight-HP layouts are unchanged. Actual damage still removes
ordinary integer HP; color is a compact representation, not an armor system.

| Full-health range | Newly added color | HP held by a full sphere at this layer |
| --- | --- | --- |
| 1–30 | Native blue | 1 |
| 31–60 | Purple | 2 |
| 61–90 | Red | 3 |
| 91–120 | Orange | 4 |
| 121–150 | Yellow | 5 |
| 151–180 | Green | 6 |
| 181–210 | White | 7 |

At 31 HP only the first sphere is purple, with two HP. At 60 all 30 are purple.
Extra HP belongs to the earliest spheres and damage drains the last occupied
sphere first. A final purple sphere goes blue after one HP of damage and empties
after the next. The same rule applies to the higher colors. Existing vortex and
shell artwork is recolored using precomputed palettes; no additional protection
spheres or draw layers are stacked. The engine now includes the white palette.
The hard usable-health limit is **210**, including all native purchases.

## Launcher/INI cap semantics

The canonical player field is:

```ini
[general]
nanotech_progression_max_health = 206
```

**An ordinary new game always starts at four HP with zero Nanotech XP.**
The setting is a ceiling; it does not grant initial health. It counts **four
starting HP plus up to 202 XP-earned HP** (valid range 4–206).
The original Premium purchase adds one independent HP; Ultra adds three more.
At the configured maximum the resulting totals are 206, 207 and 210. XP cannot
consume the four slots reserved for these purchases. The launcher explains this
in English, Italian, French, German and Spanish.

The former `nanotech_max_health` key counted XP bonus only. Its runtime parser
remains compatible with old profiles. Launcher imports/reads and installation
migrate it to the distinct canonical key once, avoiding an ambiguous silent
reinterpretation. For example, old bonus 22 becomes new progression cap 26,
retaining the same effective maximum. Old zero becomes four. Old high limits
are capped at 206. An explicit new key wins when both keys are supplied, in
both the launcher and runtime. Other values/comments/BOM/line endings are kept.

The approved default profile changes only this representation (22 → 26); all
other 2,591 values are retained. Existing player INIs and presets are migrated
inside the runtime/GUI installation rollback transaction, with backups. Native
saves and their companions are not rewritten by the installer. Previously earned
bonuses up to the old stored limit remain readable and are retained, while usable
HP is clamped to the new cap. Lowering a setting does not delete earned progress.

Internally `Settings.nanotech_max_health` remains an XP bonus cap so native shop
and progression consumers retain their established contract. Parsing the new key
subtracts the four starting HP. `MAX_XP_NANOTECH_BONUS` limits this to 202.
`NanotechProgress::valid` deliberately accepts the older stored bonus range;
the storage validity limit and the new gameplay cap are separate concepts.

## Direct local test

After compilation/installation succeeds, close the normal game and run
`04_TEST_NANOTECH_VELDIN_II.bat`. The disposable run starts on **Veldin II with
210/210 HP**, all weapons/equipment/coordinates and the upgraded ship. Premium
and Ultra are already acquired; final-level encounters remain active.

- F5 cycles maximum-health boundaries: 30, 31, 60, 61, 90, 91, 120, 121, 150, 151,
  180, 181, 209, 210, 8, then repeats. It heals to the selected maximum.
- F6 fully heals.
- F7 removes exactly one HP, stopping at one. Enemies can still damage/kill Ratchet.
- Optional initial HP: `04_TEST_NANOTECH_VELDIN_II.bat --hp 31`.

Ordinary save reads/writes are disabled for the test. The player's INI and presets
are untouched by test startup. The runtime/data paths are resolved from the
verified installation and actual launcher settings, not development defaults.

## Build/install behavior

Extract the complete v116 kit to a new folder and run `01_COMPILE_AND_UPDATE.bat`.
Select the existing **playable mod folder** (launcher EXE, weapons.ini and runtime),
not an old source checkout. No earlier source kit is required. Compilation uses
the complete bundled source.

The v115.2 cache fix is retained: validate local dependency paths, clear compiled
release artifacts for every local workspace package, rerun rc-formats/rc-game
tests and build the engine. Third-party dependency packages remain cached.
The actual executable is read from Cargo's artifact JSON. Strict v116/capability92
identity checks precede installation. The new GUI is rebuilt with the installed
custom icon and must pass its standalone self-test before replacement. This test
can take several minutes. A matching later GUI build can reuse its proof.

Runtime, matching shaders, GUI and migrated profiles share one backup/rollback
transaction. `02_OPEN_UPDATED_LAUNCHER.bat` verifies/reopens that installation;
`03_CREATE_PLAYABLE_ZIP.bat` packages the current installation and weapons.ini
without saves, personal presets or original game data. The health test is not
enabled in that player ZIP.
