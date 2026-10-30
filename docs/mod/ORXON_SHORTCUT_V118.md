# Orxon shortcut correction — local test v118

ReRAC 0.3.8-alpha; Weapon Evolution runtime patch 0.1.6.118;
configuration capability 93; launcher 0.29.6. No new configuration keys or
save-format changes. This is a local test, with no GitHub operations.

Weapon Evolution is an independent mod by TomRus47. Its author is not involved
in ReRAC's development and owns no original-game rights or assets. Thanks to the
ReRAC developers for the work that makes the mod possible; upstream licenses
and original-game rights remain with their respective owners.

## Cause and correction

The v117 matcher incorrectly required class 726, the Novalis path lift. Orxon's
copy is class **1141**, running the same relocated `PathPlatformUpdate`.
The upstream source audit records this explicitly in
`docs/plan/level_generalisation.md`, row C1; the native function cluster is
`7e8874a8` in `tools/ghidra/names/clusters.tsv` (level10 address `0x2de860`).

The matcher now recognizes the loader-resolved update function instead of
assuming that different planets share class numbers. It still requires valid
finite spline endpoints, a vertical path and an unambiguous upper endpoint near
the original Magneboots pedestal. The cached randomized source anchor is used
when available, so collecting/reusing the original pickup actor cannot lose the
reference position. Ambiguous/malformed geometry remains rejected.

Preparation now runs before scheduler culling, including the load pass. Only the
selected platform's local appear/activation gates are cleared, its update is
activated and its initial endpoint is lowered. An existing rider is never snapped.
The platform remains down during the landing sequence and until Clank boards it;
then the native movement, carrying, sound and collision behavior resumes.
This prevents native distance-based calls from raising it before it is used.

## Exact scope

All of these conditions must hold:

- Current planet is Orxon (level 10).
- Randomizer **All** is enabled and the save has an active randomized layout.
- The real O2 Mask inventory flag is already owned.
- At least one active Clank-side check (sources 28 or 75) remains uncollected.

Without the real Mask, in ordinary games, in Weapons-only mode, on other planets,
or after Clank's checks are complete, this shortcut override does not activate.
Other platforms, doors, gadgetbot puzzles, mission flags, ownership and rewards
are unchanged. Clank still plays his section before the existing Ratchet handover.
The initial-wait state is transient and cleared on death/level reload; it is not
written into the seed or save. Existing saves remain usable.

## Build and test

Extract this complete kit beside the previous v117.3 kit if possible, close the
game/launcher and run `01_COMPILE_AND_UPDATE.bat`. Choose the existing playable
folder containing the launcher, weapons.ini and runtime/rerac.exe. It compiles the
bundled v118 source; no intermediate source patches need to be applied.

The successful Rust recovery/download logic from 117.3 is retained. A verified
compiler cache in a sibling Weapon Evolution test-kit folder can now be checked
and reused, avoiding another Rust download. Cache paths are restricted to marked
kit-local roots and the actual compiler and Cargo are probed before selection.
Global Rust, persistent environment variables and security settings are not altered.

After updating, reload your save or leave and re-enter Orxon to rebuild its level
objects. No new randomizer run is needed. Verify the shortcut from Clank's starting
area with an early O2 Mask: it should wait below, carry Clank once boarded, and
leave the rest of the Clank section intact.

Automated regressions use the actual Orxon alias class 1141 and exercise the real
scheduler load/tick path, including a hidden/frozen/out-of-range platform, no XY
rider rearm, an existing rider, source actor reuse, scope exclusions and reload
state. They supplement, rather than replace, testing the user's planet assets.
