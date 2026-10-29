# 0.4.1 candidate verification

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
