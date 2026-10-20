# Verification: gameplay 0.1.6.51 / public 0.3.3

## Current v51 checks

**Release hold:** v51 / 0.3.3 is an unreleased test version. Pushes may build an
Actions test artifact, but release/draft creation requires a separate explicit
manual opt-in. No 0.3.3 release is authorized at present.

- Python launcher/release suite: **92 tests passed** locally, without failures.
- All **1,168 existing INI values** and the custom launcher icon remain unchanged; **22 controls added**, for **1,190** total entries. The new mode defaults off.
- New regression fixtures cover exact pack distributions at every Gold Bolt count, weapon rarity, last-ammo locking/re-unlocking with a full ring, permanent bolt thresholds, per-slot isolation, last-round guided Visibomb completion, normal R.Y.N.O. loot and independent Gold controls.
- Save fixtures cover legacy companion migration, v2 mode-state round trips and mode/XP synchronization without resetting combat epochs.
- Tesla dual-chain fixtures verify that secondary acquisition is silent, existing queued audio survives, and primary acquisition still sounds once.
- Rust 1.99.1, Linux x64 release: **984 game tests passed** and **154 formats tests passed, 1 ignored**.
- The linked Windows package record is pending while this revision is built.
- Required in-game checks are in [AMMO_UNLOCK_MODE.md](AMMO_UNLOCK_MODE.md). No player disc data is available to validate imported models or subjective audio/render behavior here.

The following v50/v49 records are historical, not claims that the new v51 binary has passed those exact checks.

# Historical v50 / public 0.3.2

## Current v50 checks

- Python launcher/release suite: 89 tests passed on Linux, without failures.
- Source/default/example/embedded-profile integrity: 1,168 approved INI values unchanged; custom icon byte-identical to v49.
- Rust 1.99.1, Linux x64, optimized release: **975 game tests passed**; **154 formats tests passed, 1 ignored**. The refreshed suite includes the first-frame particle-anchor assertion. The linked Windows build has also passed; see the exact package record below.
- New fixtures cover whole-volley admission, allocation rollback/two-tick quarantine, visible pellet heads under particle pressure, residual-range lifetime expiry, normal range expiry, and shared recognition equivalence/cache lifetime.
- Serialized synthetic recognition benchmark: median **4,903.198 ms before**, **115.333 ms after**, identical checksum in all six runs. This is only the repeated-recognition workload, not a planet-load speed claim. [Raw measurements](recognition-benchmark-v50.json).
- In-game validation is still pending; see [LOADING_AND_SHOTGUN.md](LOADING_AND_SHOTGUN.md).

The Windows draft workflow gates packaging on source checks, library tests,
linked compilation, runtime identity, launcher GUI self-test and packaging.
A successful build does not establish planet-load speed or validate the user's
exact saved encounter. No audio probes are enabled.

## Completed v50 Windows build — 2026-10-20

[Windows workflow run 39444201518](https://github.com/TomRus47/rerac-weapon-evolution/actions/runs/39444201518)
passed both build and draft jobs for source commit
`485a3124dea8613d8acf96795ef579642c29beeb`.

- Linked Windows MSVC release: **975 game tests passed**; **154 formats tests passed, 1 ignored**.
- Windows Python: **89 tests run, 1 platform-specific test skipped**; no failures.
- The linked runtime passed exact patch **0.1.6.50**, capability **48**, editable-profile and clean-audio identity checks.
- The frozen x64 launcher passed its Tcl/Tk GUI self-test; the supplied icon is retained.
- Player asset: `ReRAC_Weapon_Evolution_0.3.2_windows_x64.zip`, **45,794,598 bytes**.
- ZIP SHA256: `9b6fadb6b8344b7bc5da017b6b2af3e44d515d8e361cf64e624b5e4bfbfcd4ac`.
- Draft v0.3.2 includes the ZIP, SHA256 sidecar and `release-build.json` identity report. Earlier published releases were preserved.
- The prior superseded build was canceled before packaging, so the draft contains the final first-frame-anchor correction.

A separate Windows x64 engine type-check also passed using a disposable copy
with BLAKE3 pure mode enabled only there. The linked Windows workflow used the
tracked production dependency configuration. No cross-check workaround is shipped.

Keep the release a draft until the user's in-game shotgun and loading comparison
is complete. Native source tests, linked compilation and packaging are verified;
planet-load speed and the exact saved encounter have not been measured here.

---

# Historical verification: gameplay 0.1.6.49 / public 0.3.1

## Evidence and scope

The source baseline is the attached v48 update kit reconstructed using its
cumulative developer patch. The two supplied crash records have unsymbolized
stacks. A minimal v48 fixture reproduces the exact pvar panic by dispatching an
old Oltanis arc callback to a replacement empty-pvar slot. The same behavioral
fixture passes with the lifecycle fix. See RESPAWN_LIFECYCLE.md for the limit:
the exact user save has not been run here.

## Local checks

- Rust 1.99.1, Linux x64: `cargo test --release --locked --no-default-features
  -p rc-game -p rc-formats --lib`: **969 game tests passed**, **151 formats tests passed, 1 ignored**.
- Launcher and release tools: `python -m unittest discover -s launcher/tests -v`: **89 passed**.
- Source/default/example/optional embedded-profile integrity, generated reference
  reproducibility and reviewed source hashes: `python tools/mod/check_source.py`.
- Incremental updater transaction, migration, reapply and restore tests, plus an
  exercise using the actual changed source payloads and byte-exact restoration.
- Windows x64 engine type-check uses a disposable source copy with BLAKE3's
  pure implementation enabled only in that copy; no cross-check dependency or
  lockfile changes are shipped. This is not a linked Windows runtime.

The initial unoptimized Linux link failed in this environment with unresolved
compiler-generated closure shims; the normal optimized release build is the
validation target. No production workaround for that local linker issue is
included. Standard release tests must pass normally and in parallel.

## Windows build and remaining manual checks

The GitHub Windows workflow separately builds the linked MSVC runtime, runs
library/launcher tests, verifies exact runtime identity, builds/smoke-tests the
frozen launcher and packages the tracked INI/icon. Its Actions result and
release-build.json, not this local type-check, establish whether a particular
Windows package was built successfully. Draft creation is conditional on success.

Before publishing, test:

- Repeated Oltanis deaths at the reported checkpoint with Tesla, Magneboots and
  Sonic Summoner; no companion, active companion and expired companion.
- Electrical effects after respawn on Oltanis and Quartu; XP, grades, save slots,
  checkpoint location and native encounter state remain correct.
- Pause-menu XP placement on several localized names; ten-drone formations on
  open ground and near scenery; settings/presets restored in the extracted ZIP.
- Experimental shotgun release/cooldown, flame-mine landing and independent
  colors/sound, boot leg poses and steep/downward locked camera.

No game data, Windows executable or active author profile was supplied in this
source kit. Audio probes remain removed. Base-runtime rain/taxi audio is outside
this release's fixed claims. A fresh player ZIP is not equivalent to an in-game
verification of every change.

Historical v40 verification is retained in Git history; it is not a current
claim that audio diagnostics should be enabled or that old test totals apply.

## Completed Windows release build — 2026-10-20

[Windows workflow run 39416546919](https://github.com/TomRus47/rerac-weapon-evolution/actions/runs/39416546919) completed successfully for source commit
`d14687ed9a94e35be0f6ccda364bab0fff253b97`. Both build and draft-release jobs passed.

- Windows MSVC release: 969 game tests passed; 151 formats tests passed, 1 ignored.
- Windows Python: 89 tests run, 1 platform-specific test skipped; no failures.
- The linked game runtime passed exact patch/capability/clean-audio checks.
- The frozen x64 launcher passed its Tcl/Tk GUI self-test and packaging checks.
- Player asset: `ReRAC_Weapon_Evolution_0.3.1_windows_x64.zip`, 45,854,264 bytes.
- Player ZIP SHA256: `85c1aaebae9ec9acec5ea0abc3d8990db5b22bfab19eb3da70d9c94a668e9aff`.
- The draft includes the ZIP, SHA256 sidecar and `release-build.json` with the
  compiled runtime identity, source commit, profile and icon hashes.

The release remains a draft pending the in-game Oltanis/Quartu checks above.
This successful linked build supersedes any earlier statement that only a
Windows type-check had been performed; the incremental author ZIP itself still
contains source payloads, not executables.
