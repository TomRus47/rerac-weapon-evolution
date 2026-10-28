# v109 draft preparation verification

ReRAC **0.3.6-alpha**, gameplay **0.1.6.109**, capability **90**, launcher **0.29.3**. The existing public release and tag remain unchanged; the new candidate is an unpublished draft.

- Re-ran **1,329 rc-game library tests** on the restored v109 gameplay sources. Native CI caught four historical fixtures coupled to the former public balance; they now set their mode/capacity/boolean inputs explicitly. The approved INI is unchanged; all **1,329 tests passed again** after synchronization. Linux, locked dependencies, no default features and BFD linker; no native Windows game is executed here.
- **185 launcher/profile/tool tests passed**, including default-value fingerprints and draft-only release guards. **Four public API tests passed** for saved controls and lava rendering. Historical fixture assumptions about old public balance were replaced with explicit fixtures or current approved values; no gameplay settings were adjusted to satisfy tests.
- The user-supplied Windows build transcript reports **1,333 Windows game-library tests passed** and completes the v109 release build and frozen launcher check. Its SHA256-matching 0.29.3 EXE proof has all preload, repaint, eight-grade and alternate-override flags true, 17 pages, 6,208 widgets, 6,229 USER objects and 52 GDI objects. This is evidence from that supplied build, not a newly executed Windows build on this host.
- The draft workflow reruns native Windows validation and rebuilds the runtime/launcher from the exact main commit before creating and attaching player/source archives. It excludes author paths, presets and saves. Build/upload state is recorded in GitHub Actions and the attached identity report, not inferred from these preparation notes.
- Every one of the supplied profile's **2,592 values** is unchanged; `mod/default-profile.json` records the original and corrected-comment hashes plus the complete value fingerprint. Default/example/embedded reference agree.

Randomizer remains experimental and may produce blocked seeds. Model-based reachability tests are not universal physical/script/collision validation. Native rendering/scroll quality, gameplay balance and disc-specific scenarios still need playtesting. A build passing is not a claim of universal correctness.

Historical verification follows and describes its own snapshot.

# Current local v109 verification

ReRAC **0.3.6-alpha**, patch **0.1.6.109**, capability **90**, launcher **0.29.3**. Local test only; no GitHub update, release or tag. See [experimental override contracts and checks](EXPERIMENTAL_OVERRIDES_V109.md).

1,329 game library tests, four public API tests, 181 launcher/profile/tool tests and six installer tests passed. The real Tk source self-test passed for all 17 preloaded pages, all grades and five languages, including the three new controls and queued scrolling. Native Windows compilation, frozen GUI checks and in-game behavior remain author-PC verification. Personal balance, saves and presets are retained; missing experimental mine limits/durations inherit their grade's standard values.

Historical verification follows and describes its own snapshot.

# Current v106 development verification

ReRAC **0.3.6-alpha**, patch **0.1.6.106**, capability **89**, launcher **0.28.9**. Main/source update and local test kit only; no new release or tag.

## Executed checks

- **1,324 rc-game library tests passed**. The wrench regression drives the actual ground-blast query, moby grid and native Novalis enemy update at elevated terrain and world z=0: a configured 200-HP blast removes 200 HP once, awards 24 wrench XP at the fixture's explicit rate, sends nearby crates the native bomb hit and leaves a distant enemy untouched. It also binds the visual's centre/radius to that damage snapshot and rejects unbounded native blast particles.
- Visual lifetime/radius tests cover configured radii 0.1, 0.5, 1, 3, 8 and 30 units: each reaches the exact query radius, never exceeds it, retains its palette and expires. Invalid/non-finite radii do not draw. The existing Nanotech regression covers one blue XP-upgrade burst and load/death/reset suppression.
- **178 Python launcher/profile/update tests passed**, including additive/idempotent HUD migration, explicit Off retention, approved balance preservation, five-language labels/help and four v106 exact-block preflight/rollback/installation tests.
- Complete **rc-engine Rust typecheck, including test targets, passed**. The HUD regression verifies that inserted primitives retain the correct fine-coordinate/additive indices; it is compiled by this check but is not executed as a linked engine test on this host.
- The reviewed source hash index, generated configuration reference and source-export checks are refreshed. The delta ZIP is recreated from tracked source, verifies its payload hash and passes ZIP integrity checks.

## Environment and limits

Library tests use Linux Rust, the locked dependency graph, no default features and the BFD linker. Native Wayland/ALSA/udev SDK metadata is unavailable; the engine check uses a temporary, typecheck-only pkg-config stub. This proves Rust types, **not native availability, ABI compatibility or a linked Windows/Linux engine**. The stub is not shipped or committed.

The v106 Windows updater requires installed v105 sources and reruns the actual game tests/native release build before replacing the playable runtime. It retains the confirmed **0.28.9** launcher, INI, presets, saves and icon. A full repository checkout is built through the normal root scripts instead. In-game appearance, damage-boundary perception, lighting and native Windows execution still require playtesting. The user confirmed damage, fire and HUD fixes from the previous local build; v106's visual change is newly supplied for testing.

Historical verification records follow; their counts and versions describe their own snapshots.

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
