# 0.4.2 candidate verification — 2026-10-30

Runtime base **ReRAC 0.3.8-alpha**, gameplay **0.1.6.123**, capability **93**,
launcher **0.29.6**. The author supplied a complete prepared v123 kit: every one
of its **1,416 inventoried source files** matched its SHA-256 before import.
Unlisted Python caches, kit logs and installation state were excluded. The
current main-branch reward/vendor/race fixes were retained during reconciliation.

- **1,422 rc-game library tests passed**, locked offline dependencies, no default
  features, release profile with host optimization disabled. This includes the
  full thousand-layout randomizer survey and the actual race-command, reward,
  persistent-vendor, shortcut, retained-enemy and burst-tracking tests.
- **223 Python tests passed**, covering profiles, localization, source identity,
  scoped Rust recovery, packaging and draft-only release behavior. The final run
  used matching development/draft descriptors; an earlier run during descriptor
  preparation is not counted as a successful validation.
- Direct comparison of unnormalized INI documents confirms **all 2,592 existing
  default settings are unchanged**. The only added key is the neutral
  general.enemy_burst_tracking = false. The default/example, optional embedded
  profile and default-profile fingerprint remain consistent.
- The supplied Windows v123 log independently records 162 passing format tests
  (one ignored), 1,422 passing game-library tests, native rc-engine linking and
  actual EXE patch 123/capability 93. It describes the supplied local build, not
  the later release commit's independent build.

The checked draft workflow reruns Windows tests, builds the runtime and frozen
launcher, and attaches player/source ZIPs, checksums and release-build.json from
one exact commit. Its completed run is authoritative for those binaries. These
host results do not claim a GPU playtest or universal randomizer solvability.
Native saves, matching companions, seeds, placements and personal profiles are
preserved. No game assets or player save/log files enter the source export.

Historical local verification follows.

---

# v118 local test verification

ReRAC **0.3.8-alpha**, gameplay **0.1.6.118**, configuration capability **93**,
launcher **0.29.6**. Changes: [Orxon shortcut](ORXON_SHORTCUT_V118.md).
No GitHub operations. Balance, launcher implementation and save formats remain
unchanged from v117; the existing 117.3 Rust download recovery is retained.

The v118 regressions use Orxon's real alias class 1141 through the scheduler,
not only a Novalis-class synthetic fixture. Scope checks cover absent O2,
ordinary games, Weapons-only mode, another planet, inactive layouts and completed
Clank checks. The hidden lift starts below, waits until boarded, and retains
native movement afterward. An existing rider is not snapped; transient state
resets on reload. Source-anchor fallback survives pickup actor reuse.

Python verification also covers reuse of a verified compiler cache from a sibling
mod test kit, avoiding another download after the successful 117.3 recovery.
**1,396 rc-game library tests passed**, with no warnings.
**223 Python tests passed**; the updated marked-cache filter also passed its
eight focused build-tool tests. Source inventory, source hashes and kit
preflight checks are run again on the exported package.
The host still lacks Windows/GPU/game assets and full engine dependencies, so
Windows runtime linking, GUI execution and the user's actual Orxon scene must be
verified by the local installer and gameplay test.

## Inherited v117 verification and installer history

# v117 local test verification

ReRAC **0.3.8-alpha**, gameplay **0.1.6.117**, configuration capability **93**,
launcher **0.29.6**. This package is a local source test; no GitHub push or release
operation was performed.

- **1,393 rc-game library tests passed**, with no Rust warnings. New coverage
  includes rail collection policy, retained Kalebo robot lives, fixed-room camera
  scope/aim, Orxon shortcut isolation, burst launch vectors and wrench XP timing/caps.
- **222 Python tests passed**, covering launcher schema/localization, neutral
  migration, balance preservation, installer integrity/rollback and packaging.
  Release upload tests use mocks and do not contact GitHub.
- Source inventory and reviewed hashes passed `tools/mod/check_source.py`.
  The 2,592 preceding author-supplied values remain unchanged; the only additional
  default is `enemy_burst_tracking = false` (2,593 values in 146 sections).
- Complete source export is integrity-checked against its bundled SHA-256 map;
  native binaries, private settings, saves, build outputs and disc assets are excluded.

Host limits: rc-engine checking could not start because the host's offline Cargo
cache lacks engine dependencies (first missing package: ab_glyph). This host has
no Windows GUI/display or extracted game assets. Consequently this is not a
Windows runtime/GUI build or a gameplay verification. The local installer runs
fresh Rust checks, compiles the exact bundled engine, verifies its reported EXE
identity, and tests the standalone GUI before replacing the playable installation.
The Orxon geometric lift match needs the in-game check described in
[COMBAT_PICKUPS_V117.md](COMBAT_PICKUPS_V117.md); ambiguous geometry fails closed.

Installer revision **117.1** fixes only the stale preflight version check.
Five new tests cover the exact pre-Cargo validator and export guard. Gameplay
and launcher source match the first v117 kit byte for byte.

Installer revision **117.2** adds seven regression tests for compiler
preflight/recovery, verified-cache reuse, environment isolation and blocked
execution. The metadata probe also passed against an actual host Rust compiler.
Downloading/executing the separate Windows toolchain still requires the user's
Windows test; automated failure/recovery tests use controlled subprocess mocks.

Installer revision **117.3** adds seven transport-fallback tests, including an
actual child process stream/exit check. Network success/failure cases use
controlled subprocess mocks; connectivity from the user's Windows network and
support for the optional cURL backend cannot be established on this host.

## Previous candidate evidence


Current source: ReRAC **0.3.8-alpha**, gameplay **0.1.6.116**, capability **92**,
launcher **0.29.5**. The v116 local validation passed **1,382 rc-game library
tests** and **201 Python tests**; rc-engine test targets passed host type checking.
These results are inherited source-validation evidence, not a Windows/GPU playtest.
The draft workflow reruns checks and builds native Windows binaries from the exact
new commit. Its run and release-build.json are authoritative for that build.

The release default comes exclusively from the attached player ZIP's root
weapons.ini. All authored settings are preserved; the compatibility comment alone
is updated to 0.3.8-alpha. mod/default-profile.json records raw/public/value hashes.
The embedded profile and generated reference use these same defaults.

No test environment is active in normal launcher startup: new games start at four
HP. The Veldin II 210-HP test requires its explicit test entry point and does not
write normal saves. Randomizer solvability remains experimental; see the release
notes for the save-report procedure. No publication is performed automatically.

Historical verification follows.

---

# v116 local verification — 2026-10-29

Input: the author's current working v115 source export. All **1,394** inventoried
source files were SHA-256 verified before editing. This source includes the
v115.2 workspace-cache fix and the earlier pickup-completion fixes.
No GitHub repository, draft, release, tag or remote asset was changed.

- **1,382 rc-game library tests passed**, locked dependencies, no default features,
  release profile. This includes the complete thousand-layout randomizer survey.
- **201 Python launcher/installer/packaging tests passed.** The profile migration
  also passed its focused four-test suite, including old integral decimal/number
  representations, comment/BOM/CRLF preservation and idempotence.
- **Three cap/progression tests passed again** after adding the explicit initial
  health assertion: a fresh zero-bonus progression remains at native health,
  including exactly four HP with no Premium/Ultra purchases. A configured ceiling
  never grants initial HP. The special local test separately starts at 210/210.
- `cargo check --release --locked --no-default-features -p rc-engine --tests`
  passed against the complete ReRAC 0.3.8 workspace. This Linux-only typecheck
  used a host pkg-config stub for unavailable native SDKs. The stub is outside
  the source and is not distributed. This is not Windows linking or GPU testing.
- All 2,592 approved default settings were compared to the uploaded baseline.
  Only the cap representation changes: old XP bonus 22 becomes base-plus-XP cap
  26. The other 2,591 values remain identical. The default-profile hash record and
  generated configuration reference reflect that explicit migration.
- Installer fixtures verify that runtime/GUI/profile updates roll back together
  on failure, with source/default bytes, presets and ordinary saves retained.

The new runtime regressions exhaust every current/max HP pair through 210,
verify at most 30 spheres, color boundaries through white, exact HP conservation,
alpha/highlight preservation, native purchase reservations, the 206/207/210 totals,
old high-bonus save round trips and canonical-key precedence in either order.
The launcher regressions check 4–206 bounds, all five UI languages, old-profile
conversion and the private save-disabled test environment.

The complete kit includes a full source inventory and verifies it before build.
Its Windows installer retains the v115.2 clean-local-packages policy, reruns fresh
Rust tests, verifies Cargo's actual engine artifact and rebuilds/verifies the GUI
before installing any changes. It does not rely on the old shared-cache results.
The final ZIP is CRC checked and every bundled source hash is compared again.

In-game visual validation requires the player's own data and Windows runtime.
Use the kit's 04_TEST_NANOTECH_VELDIN_II.bat after successful installation. Normal
new games start at four HP. Only the explicit disposable test starts at 210/210.
No audio probes were added.

Weapon Evolution is an independent mod by TomRus47, not a ReRAC development
project. The author owns no original-game rights or assets. Thanks to ReRAC's
developers for their work. Existing licenses and contributor credits apply.
