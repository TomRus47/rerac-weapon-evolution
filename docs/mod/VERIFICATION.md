# Current v102 development verification

ReRAC 0.3.6-alpha, patch 0.1.6.102, capability 88, launcher 0.28.7. Main/source update and local test kit only; no new release or tag.

## Executed checks

- **1,319 rc-game library tests passed**. They cover accepted-loss tint, bounded fire cycles independent of stagger, native creatures without XP adapters, identity reuse, independent wrench slam/explosion attribution, eight-grade parsing, HUD row geometry/visibility and exact five-medium-enemy NG+ duplication with doubled difficulty.
- **14 public API tests passed**: controls (2), golden reward/ammo contracts (4), encounter rendering (1), lava rendering (2), randomizer displays/live configuration (5). Legacy fixture assumptions about guaranteed early power weapons and Survival bypassing randomized heavy-ammo rarity were updated to the supplied v100/v101 contracts.
- **173 Python launcher/update tests passed**: 168 launcher/profile/build checks and five local-update checks. These exercise additive/idempotent profile migration, existing-value retention, localization, field ordering, successful installation to the actual EXE, runtime-file retention, rollback after compilation or installed-GUI verification failure, and unknown/escaping payload rejection.
- The machine-readable route snapshot and CSV were regenerated from the current production catalog via `randomizer_audit --catalog`. No saved layout or personal profile is regenerated.
- The configuration reference, reviewed source hashes and source-export checks are refreshed for this snapshot. The public default retains all prior approved values; new optional controls default Off.

## Environment and limits

The library and API checks use Linux Rust, the locked dependency graph, no default features and the host-compatible BFD linker. The complete rc-engine Rust typecheck passes. Native Wayland/ALSA/udev development packages are unavailable on this host, so the typecheck uses a temporary pkg-config metadata shim; it verifies Rust types, **not native library availability, ABI compatibility or a linked Linux/Windows executable**. The shim is not included in source or the test kit.

The Windows updater runs the actual game tests and native Windows release build, then rebuilds and checks the frozen launcher before replacing the selected playable EXEs. The frozen self-test renders and toggles all eight grades' four new wrench Off/On controls. That native build/GUI proof and in-game rendering, sound and controller checks must run on Windows; they have not been executed on this Linux host.

Before release, playtest continuous fire and feedback on several enemy types, independent wrench slam damage/XP, death/reload cleanup, both HUD positions and Triangle-only wrench display, swimming, NG+ duplication, and crate/Infobot/Gold Bolt models. Test the existing personal INI/presets and approve the next default profile. Route-model reachability is not universal physical/script validation.

## Historical records

# v95 route audit verification

Current checks and remaining limitations are documented in [RANDOMIZER_ROUTE_AUDIT.md](RANDOMIZER_ROUTE_AUDIT.md). This main update is not a release.

## Executed v95 checks

- 1,287 rc-game library tests, two controls API tests and five randomizer API
  tests passed (1,294 combined). New checks bind the reported Blarg IDs, reject
  the old blocked permutation, permit Ratchet's return without his own Gold Bolt,
  verify all 101 entries and retain old save decoding.
- 1,000 sequential seeds passed exact reward validity, reachability and pacing
  through an optimized harness compiled from the production generator/catalog.
  Boundary/reported cases 0, 87, 478, 723, 14461498566715420534 and u64::MAX passed.
- Full rc-engine check passed with the supplied complete ReRAC 0.3.6-alpha source.
- 165 launcher/tool checks passed, including current schema/capability handling,
  preset preservation, default migration stability and per-grade mode limits.

The exported catalog records every check's review basis. Unknown native Gold Bolt
identities use conservative envelopes; no claim of universal physical validation
is made. A live route/collision/script survey still requires original extracted
data and in-game verification. Old saves load without automatic rearrangement.

## Historical validation record

# Verification — current main development snapshot

ReRAC 0.3.6-alpha, gameplay 0.1.6.83, capability 84, launcher 0.27.1. Main/source update only. No release has been created or replaced for this snapshot.

## Completed

- **1,233 rc-game library tests passed**, zero failures or ignored tests.
- **7 public API integration tests passed**: saved controls (2), golden reward registration/contracts (4), encounter render API (1).
- **164 launcher/build-tool tests passed**. Historical fixtures were updated for the supplied eight-grade profile and current capability, retaining balance, profile, packaging, restoration and publication-guard checks.
- Source export validation passed: approved roots, no generated executables/disc data/private paths, default/example equivalence, embedded-profile synchronization, source-map hashes, current runtime identity, required workspace crates and Git ignore rules.
- The configuration reference was regenerated from the current reviewed profile and schema. Personal configuration files are not tracked.

The current regression suite covers physical D-pad/analog separation, radial input locks, native equipment requests, both equipment pages, all fourteen gadget-grid entries, ordinary confirmation, boots/helmet repeat toggles, backpack first selection, cycle/wrap/ownership, rapid pending requests, external changes, both crouch shoulders, R2/Circle unions, per-slot v8 companion persistence and legacy v1–v7 migration.

## Environment and remaining checks

Host-side Rust validation used Linux Rust 1.99.1, locked/offline dependencies, `--release --no-default-features` with opt-level 0 and one codegen unit to fit the environment. Core gameplay/API checks compiled the current source export, including its synchronized optional embedded profile fixture. Python tests used the host Python 3.12 runtime; Windows builds use their selected x64 authoring Python.

The complete rc-engine Windows link, actual frozen Tk GUI, rendering, controller feel and confirmation audio have not been exercised in this environment. The local host lacks Windows tools and uncached Bevy dependencies (an offline engine check stopped at `ab_glyph 0.2.32`). Run the documented Windows build and smoke checks and test in game before preparing a release.

## Gameplay checks before release

- Bind and equip weapons and every gadget category; check outlines, Weapons-only caption, cursor freeze and native context restrictions.
- Toggle equipped boots and helmets repeatedly, including during animations. Select a Clank pack, cycle all owned packs, test missing packs, pause-menu changes, death, travel and another save slot.
- Check L1 and R1 crouching in normal/locked movement, R2/Circle fire and contextual radial R1 mode cycling without a gameplay crouch.
- Verify supported weapon modes, unchanged launcher-only enhancements, ammo capacities, XP and normal confirmation audio.
- Recheck discovery seeds/locations, NG+ purchase/grades/rewards, population and Nanotech progression on the migrated runtime.
- Compile both EXEs, run the frozen self-test, package locally and test the extracted standalone folder with the current modified runtime.

Publication remains deferred until the author finishes testing. `mod/release.json`, published release tags and existing assets are not advanced by this main update.
