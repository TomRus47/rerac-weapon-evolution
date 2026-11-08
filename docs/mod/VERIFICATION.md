# v140 final-tick release recovery and half-unit placement verification

- 1,521 rc-game library tests passed; three unchanged large historical seed
  surveys were filtered out. Five new regressions cover jump/running-jump/
  double-jump/hover handoff to native fall with retained motion; recovery after
  an actual deferred native ItemCall pass; complete Game ticks with a later
  native idle request; synchronized body/back palettes; hand attachment refresh
  without a second animation advance; and recovery cancellation on landing,
  damage, native swing and a new jump press.
- The complete Game fixture runs the ordinary hero, native hand/deferred item,
  custom gadget and final-publication paths over twenty ticks. It checks a valid
  evaluated pose each frame, continued descent, and Clank's cached chain against
  the same final pose. This extends the narrower v139 HeroUpdate-only fixture.
- All 295 Python launcher/tool tests passed, including complete v140 export,
  exact runtime identity, CRLF scripts and source-integrity validation.
- The native capsule-contact regression now checks the requested 0.5-unit
  additional clearance through repeated complete victim rotations. Existing
  collision restoration, accepted HP/Nanotech XP, capture-owned motor and Hoven
  frost one-shot tests remain passing.
- All INIs, approved default profile, save format, randomizer generator, launcher
  schema/languages/core/icon, release metadata, manual-only draft workflow and
  native frost-audio importer are byte-identical to supplied v139. The release
  marker is transient hero state, not a new save/config field.
- Every source hash and source-map entry is checked against an independently
  extracted kit, with prepared source API/identity validation. Windows engine
  linking, real disc assets, GPU appearance and audible output still require
  local testing; no visual in-game playthrough is claimed by this host.
- Main synchronization creates no tag, release, draft or GitHub asset. Existing
  original-game rights notices, ReRAC credits and licenses are retained.

Earlier records below retain their version-specific scope.

# v139 airborne release and contact acquisition verification

- 1,516 rc-game library tests passed; three unchanged large historical seed
  surveys were filtered out. Six new regressions cover decoded fall recovery
  after release, missing-source repair when native fall is already sequence B,
  real hero falling updates with Clank's cached back chain, no acquisition for
  nearby/distant/stale unconfirmed contacts, selecting the actually contacted
  enemy, and native capsule collision followed by sustained capture at the
  one-unit-forward model centre through repeated full rotations.
- All 291 Python launcher/tool tests passed, including complete v139 export,
  exact prepared runtime identity, CRLF scripts and source-integrity checks.
- Earlier focused tests model a contact notification explicitly; the new
  capsule fixture drives the native collision resolver against a real moby
  primitive and passes its resulting cap_moby/cap_hit to item acquisition.
- Palette tests first establish the missing-source identity-matrix fallback,
  then check valid non-identity matrices after recovery. The native hero test
  checks continuing descent and Clank's attachment over fifteen airborne ticks.
- v138 collision ownership, accepted HP/Nanotech XP and motor/frost sound tests
  remain passing. Audio importer/engine files are unchanged by this update.
- All INIs, approved defaults, save format, randomizer generator, launcher schema,
  language catalogs, core/icon, release metadata and manual draft workflow are
  byte-identical to the supplied v138 baseline. No save/config fields are added.
- The independently extracted complete kit is checked against every source
  SHA256 and the prepared source API validator. Windows engine linking, actual
  disc assets, GPU alignment and audible output still require the local test;
  no visual playthrough is claimed by the preparation host.
- Main synchronization creates no tag, release, draft or GitHub asset. Existing
  original-game rights notices, ReRAC credits and licenses are retained.

Earlier records below retain their version-specific scope.

# v138 capture collision and frost audio verification

- 1,510 rc-game library tests passed; three unchanged large historical seed
  surveys were filtered out. Eight new regressions cover temporary physical
  collision ownership, original false/mask preservation, native collision-query
  rejection while held and acceptance after release, empty/captured motor audio,
  defeat/recycled-slot cleanup, jump/unequip/disabled/death interruptions,
  world-tick cleanup, accepted HP/Nanotech XP with collision absent and one native
  Hoven sound event at the actual frost impact per detonation.
- All 287 Python launcher/tool tests passed, including full-source v138 export,
  runtime identity, CRLF scripts, retained GUI and source-integrity checks.
- Audio tests use a SoundSink implementation that allocates a voice slot and
  records its release, rather than treating a refused play as an active loop.
  The frost test checks exact class/index/flags and position, freezes a real
  enemy and confirms that no actor is allocated to own the one-shot.
- Existing native audio importer tests pass. Cross-planet stock Hoven bank
  availability, audible output and engine linking require the included Windows
  test; the host has no disc assets, GPU playthrough or Windows linker.
- All INIs, approved default descriptor, save-format code, randomizer generator,
  launcher/icon, public release metadata and manual-only draft workflow remain
  byte-identical to v137. No new launcher fields or save schema are introduced.
- Exported source is checked by SHA256, including a separately extracted kit;
  the source API validator also checks the prepared engine identity/interfaces.
- Main synchronization creates no tag, release, draft or GitHub asset. Existing
  original-game rights notices, ReRAC credits and licenses are retained.

Earlier records below retain their version-specific scope.

# v137 gadget input, animation and pivot verification

- 1,502 rc-game library tests passed; three unchanged large historical seed
  surveys were filtered out. Eleven new regressions cover native walk/run pose
  ownership, retained jump/damage poses, movement-only latch filtering, vertical
  collision-resolved pull, the real low animated item chain, fixed capture
  centres with mirrored/scaled models, authored-row restoration, stable decoded
  aim keys, Circle/R2 equip input, repeat intervals and masked menu handoffs.
- All 283 Python launcher/tool tests passed, including complete standalone v137
  export with CRLF batch scripts, runtime identity checks, preservation of the
  existing launcher and rejection of incomplete/modified source inventories.
- Native palette regressions from v136 still pass, including missing-snapshot
  repair and agreement between Ratchet's final pose and Clank's cached chain.
- Source-only gadget changes add no saved fields and do not alter damage/XP
  accounting. Trespasser still kills through accepted native damage without
  allocating a replacement actor. Freezer ammo/recharge remain separate from
  the transient equip-input latch.
- All INIs, approved default descriptor, save-format code, randomizer generator,
  launcher/icon, published release metadata and manual-only draft workflow are
  byte-identical to the supplied v136 baseline. The test kit builds the runtime
  only; the existing 0.30.2 editor exposes all unchanged controls.
- Windows engine linking, actual disc assets, GPU appearance and Tk painting
  cannot be verified on this host. Native logic/geometry tests and Python tool
  checks are not a claim of an in-game visual playthrough. Use LOCAL_TEST_V137.md.
- Main source synchronization creates no tag, release, draft or GitHub asset.
  Existing credits, original-game rights notices and licenses are retained.

Earlier records below retain their version-specific scope.

# v136 grapple and Trespasser verification

- 1,491 rc-game library tests passed; three unchanged large historical seed
  surveys were filtered out. Five new regressions exercise actual palette
  evaluation, incomplete decoded sequences, missing snapshot recovery, the native
  hero update/back-attachment cache and a confirmed kill without replacement.
- All 279 Python launcher/tool tests passed, including full-source v136 export,
  translated help, retained Freezer settings and the checked Windows GUI workflow.
- Palette fixtures first establish the identity-matrix fallback for a missing
  source snapshot, then verify a real non-identity evaluated pose after recovery.
  Hero-update fixtures compare Clank's cached back position to the same final
  animation chain used by the renderer over twenty native movement updates.
- Existing drill tests now verify no allocation after a confirmed defeat; native
  corpse visibility/collision remain owned by the class death update. The former
  clone spawner, projectile runtime and stored corpse model are removed.
- INIs, approved default-profile descriptor, save-format implementation, seed
  generation and custom icon are byte-identical to verified v135.
- Main is synchronized with the full local development history; draft builds
  require explicit workflow_dispatch. No tag, release, draft or asset upload to
  GitHub is performed by this update. Existing public release metadata is retained.
- These are native logic/geometry and Python checks. Windows engine linking,
  real game assets, GPU appearance and Tk painting still require local/Windows
  validation; no visual in-game playtest is claimed by this preparation host.

Earlier records below retain their original version-specific scope.

# v135 gadget poses and Freezer verification

- 1,486 rc-game library tests passed, including nine new pose/grounding,
  recharge, condensation and HUD regressions. Three unchanged large historical
  seed surveys were filtered out. All 26 focused gadget tests passed.
- All 275 Python launcher/tool tests passed. These cover additive/idempotent
  migration, bounds, ordering before colors, localized labels/help, capability
  checks, self-test metadata and full-source kit export with GUI rebuilding.
- Animation fixtures use a real RatchetAnim binding with missing Swingshot
  sequences and with one-frame sequences; the decoded pose remains available
  and entry frames remain in bounds. Native jump/capture fixtures verify no
  airborne sound/capture/standing-pose requests and release on leaving ground.
- Recharge fixtures cover exact intervals, duplicate tick calls, full-capacity
  credit, disabled/dead/cutscene timing, capacity changes, checkpoint retention
  and a new planet visit. HUD fixtures verify the current/max count, icon-free
  native ammo panel, shared fade and the full yellow weapon-style bar.
- Condensation fixtures do not provide a particle pool or texture definitions:
  emitted geometry follows a real frozen actor and vanishes at thaw. The generated
  texture has a soft alpha boundary and opaque core in native GS alpha units.
- Every pre-existing INI value is preserved. The sole added setting is
  gadget.hydrodisplacer/recharge_seconds=10. Save-format code, seed generation
  and the custom icon are byte-identical to verified v134.
- A host rc-engine check could not proceed because the offline dependency cache
  lacks ab_glyph 0.2.32. No successful engine type-check/Windows link, Tk painting,
  real asset playback or GPU/gameplay test is claimed here. The local Windows
  installer performs native tests, engine linking/version probing and the GUI
  self-test before committing playable replacements.

Earlier records below retain their original version-specific scope.

# v134 local gadget verification

- 1,477 rc-game library tests passed, including eight new lifecycle/physics
  regressions. Three unchanged large historical seed surveys were filtered out.
  The focused gadget run passed all 17 tests.
- All 268 Python launcher/tool tests passed. The historical v106 updater fixture
  now uses its own immutable manifest interfaces rather than assuming the current
  renderer still matches v106; rollback/idempotence checks remain exercised.
- Matrix tests use real scheduler matrix rebuilding, a live class-459 Novalis
  trooper over 180 frames, allocation replacement and a confirmed death followed
  through projectile expiry. They distinguish a live actor, despawn and defeat.
- Swingshot checks exercise a Novalis trooper pull over 45 matrix updates and the
  actual L1 hero physics path over 20 frames; upward travel remains active and
  release disables the custom step. Native target priority remains covered.
- Hydro tests start with a below-floor hand socket at nonzero world coordinates,
  use a real floor-triangle query, then verify a ground impact, captured radius,
  two frozen Novalis classes and leases surviving repeated matrix rebuilding.
- All INIs, the default-profile descriptor, non-test launcher files, save decoder
  and randomizer generator are byte-identical to verified v133. The existing
  launcher 0.30.0 remains compatible; no source or playable file is changed before
  the Windows build/identity checks succeed.
- No Windows rc-engine link, real model playback, Tk painting or GPU/gameplay test
  is claimed on this host. See LOCAL_TEST_V134.md for the remaining visual checks.

Earlier verification records below retain their original version-specific scope.

# v132 local verification

- All 1,458 rc-game library tests in the final run passed, including the five
  new receipt/pricing regressions and the 64-mask location-generation check.
  Three unchanged large historical seed surveys were filtered out.
- All 253 Python launcher/tool tests passed, including the new complete kit export.
- Balance INIs, seed-generation code, save-format code and all 27 non-test launcher
  files were checked byte-for-byte against the verified v131 input and retained.
- Fixtures exercise a real randomized Metal Detector grant, closing-vendor world
  frames, movie-mode suppression, one-shot receipt removal, and a repeated source
  grant that cannot enqueue the reward a second time.
- Talk fixtures run the actual NPC update through denied and accepted purchases:
  499/500/501 bolts for Novalis, and 14,999/15,000/15,001/149,999/150,000 for Rilgar
  in normal and All modes. Price tables remain byte-identical. All 43 payment
  records are checked for the scoped RYNO exception, alongside all six game
  language IDs and free/trade offer handling.
- No Windows rc-engine link, real movie playback or GPU/gameplay test is claimed
  on this host. The local installer runs its Windows build before replacing the
  playable runtime. See LOCAL_TEST_V132.md for the in-game skip/completion checks.

Earlier verification records below retain their original version-specific scope.

# v131 Hoven fallback verification

The two new native fallback/save-compatibility regression tests passed. The
complete launcher/tool suite passed all 249 Python tests, including the five
localized fallback descriptions and full GUI/runtime export checks.

See LOCAL_TEST_V131.md for generation-only normalization and compatibility with
existing v130 zero-site saves. Earlier verification records below retain their
original scopes. No Windows game/GPU test is claimed by this preparation host.

# v130 local verification

- 1,450 rc-game tests passed in the main run; four tests were filtered there.
- The 64-mask extra-site generation/solvability/serialization test passed separately,
  bringing coverage to 1,451 distinct passing tests out of 1,454. Three unchanged
  large historical seed surveys were not repeated. Route checks model requirements,
  not every physical/cinematic path in the original disc.
- Launcher/tool verification covers 244 Python tests, including all five UI
  languages, profile preservation, live/restart policy, staging and full GUI/runtime
  kit export.
- Actual Taunter lure delivery was checked in both launcher and saved R1 modes:
  pure mode keeps attraction/XP and queues no damage, stun or launch.
- PDA refill tests cover remote prices, insufficient funds and exact affordability.
- Save regression coverage includes v1-v10 loading, v11 round trips, unchanged
  layout offsets, previous R1 choices, both rings, seeds and single-claim packs.
- Blarg native AI timers advance at 1, 1.25, 2 and 4 times effective NG+ speed.
  This does not reproduce or claim to fix all stationary-enemy reports.
- Localized name/XP presentation fixtures cover all weapon IDs at V6-V8 and
  English, Italian, French, German and Spanish bytes, plus native gold-name fallback.
- The preparation host has only the rc-game/rc-formats dependency set. It does
  not perform the native Windows rc-engine link, frozen GUI or GPU visual test.
  The bundled installer runs the Windows build and frozen GUI self-test before
  replacing installed executables. Source/ZIP integrity is checked separately.

See LOCAL_TEST_V130.md for player checks and source changes. Earlier verification
records below describe their original test scopes; they are not new v130 results.

# Local v129 Quick Select audio and liquid-surface verification

Base: the user-supplied v128 prepared source archive. All 1,424 inventoried file
hashes matched before import. Runtime 0.1.6.129 / ReRAC 0.3.8-alpha, capability 94;
launcher 0.29.7 retained. No GitHub operation, Windows deployment or release edit.

- **1,441 rc-game library tests passed**, 0 failed; 3 unchanged expensive seed
  surveys excluded from this repeat. The new synthetic audio regression
  reproduces START_PENDING after IOP-only output and verifies immediate EE start
  while no game frame runs, no extra samples are generated, and no start is
  duplicated on the following update. The final test total is 1,444.
- **234 Python launcher/build/export tests passed**, including 11 compiler-cache
  and link/run-probe tests, 8 runtime installer tests and 4 standalone source-kit
  export tests. The export includes every required helper and rejects changed or
  incomplete inputs before producing a ZIP; unlisted private files stay out.
- **rc-engine `cargo check --tests` passed** with locked dependencies and no
  default features on the Linux host. Native dependency metadata was extracted
  to the workspace only. This compiles the full engine interface and test code;
  it is not a Windows linker or GPU/gameplay run.
- The source inventory preserves every existing INI and ICO byte. Launcher
  application Python modules remain byte-identical to v128. Only its build-tool
  tests change; no GUI reconstruction or language/schema migration is required.
- No game data, player saves, executable binaries, installation records or
  personal logs are included. The complete kit has its own source SHA inventory
  and builds without an older patch folder or a Git checkout.

In-game confirmation remains necessary: audible R2/R1 confirmations while
Triangle is held, no replay after release, and the remaining Pokitaru purple
water surfaces under normal fog. World freezing and above-water fog must remain
intact. See [local test instructions](LOCAL_TEST_V129.md).

Earlier verification records follow; their version numbers describe those
historical snapshots and do not replace the v129 inventory.

# Local v125 dual Quick Select verification — 2026-11-04

Base: complete main source 0dd5ad997a8678a8837a9af23e4ada0c931bd41c,
including the local v124 fixes below. Runtime 0.1.6.125 / ReRAC 0.3.8-alpha,
capability 94; launcher 0.29.7. Local source/test kit only; no remote mutations,
release edits or publication. No player saves or installed Windows files accessed.

- **1,435 final rc-game library tests passed**, locked offline dependencies, no
  default features, release profile with host optimization disabled. Three
  unchanged, expensive pure seed surveys were excluded from this final repeat;
  all three already passed in the complete 1,436-test v125 run before the final
  R2-pressure and second-ring ownership guards. Generator/placement logic is
  unchanged. The final run includes both editor banks, independent ring ammo and
  equip, held-button edges, empty-page drawing, all eight Pokitaru gadget subsets,
  Survival re-unlocks, legacy v1–v9 readers, v10 validation/slot isolation and
  progress updates retaining live XP/combat epochs.
- **227 Python launcher/tool tests passed**; **8 focused updater tests passed**.
  Migration preserves BOM/CRLF/comments and every previous value, default Off,
  five-language metadata, older-runtime refusal when pause is enabled, rollback,
  path-with-spaces handling and executable verification.
- **rc-engine `cargo check` passed** on the Linux host with locked dependencies,
  no default features and workspace-local build-time pkg-config metadata. This
  type-checks the actual Bevy integration, separate paused menu input, world tick
  gate, render-clock gates and mode/reset paths. It is not Windows linking or a
  GPU/gameplay test. No dependency, Cargo.lock or OS installation was modified.
- All **2,593 preceding default INI values** match main byte-for-value; the only
  new setting is `pause_on_quick_select=false`. The supplied icon is unchanged.
- Source export checks validate 1,422 tracked files and the reviewed mod hashes.
  The local ZIP contains all source, an exact file hash inventory and three
  Windows build/open/package scripts; no EXEs, game/disc assets or private files.
- Windows compilation, the frozen Tk/EXE self-test and visual game checks must
  still run on the player's Windows PC. The updater requires a successful actual
  Off/On field-render test before replacing the launcher. Test instructions are
  in [LOCAL_TEST_V125.md](LOCAL_TEST_V125.md).

# Local v124 combat/render verification — 2026-11-04

Base: complete main source 0dd5ad997a8678a8837a9af23e4ada0c931bd41c on
ReRAC 0.3.8-alpha. Local runtime identity 0.1.6.124, capability 93; launcher
0.29.6 and approved INI/default/icon bytes are unchanged. No GitHub mutations,
draft edits, release publication or player-save changes were performed.

- **1,426 rc-game library tests passed**, locked offline dependencies, no default
  features, release profile with host optimization disabled. This includes all
  damaging weapons/amoeboid sizes, special actions, zero/custom XP rates, health
  and reward scaling, split descendants, intact crate poses/bounds, canceled
  capture, cannon attribution and Veldin's actual beam-manager clock, plus the
  retained mission/vendor/race and full randomizer layout survey.
- **224 Python tests passed**, including the new runtime-only entry-point test,
  which installs the verified runtime while retaining GUI, INI and presets.
  The focused eight-test installer suite passed independently too.
- Source-export checks and the real extracted kit's inventory/preflight pass.
  The ZIP contains complete reviewed source and three local build/open/package
  entry points, excluding build caches, private settings, logs, saves and disc
  data. Both public and embedded defaults and the custom icon retain their bytes.

See [behavior and local test instructions](LOCAL_TEST_V124.md). These are headless
source tests, not native Windows linking or a GPU playtest. The local installer
reruns Rust tests, compiles and checks the actual Windows EXE before replacing
the installed runtime/shaders transactionally. GPU appearance still needs the
player's in-game check; no Visibomb tuning is changed.

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

The first draft CI attempt exposed Windows TEMP's short 8.3 alias in three Rust
recovery fixtures. The fixtures now resolve their roots, matching production's
existing canonical paths, including the sibling-cache mock. Production recovery
and gameplay code are unchanged. Draft build revision 2 reruns those checks;
the failed first attempt did not create or publish a release.

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

## v133 — optional gadget combat and continuous lava

The v133 source was derived from the user-attached, SHA-verified v132 kit.
1,469 rc-game library tests passed, with no failures. Three unchanged large
randomizer seed surveys were filtered from the final run; the 64-extra-mask
progression generator regression and existing save round trips were included.
New native coverage checks gadget settings, target/pad priority, floor rejection,
ordinary enemy versus boss/ship pulling, ice ammunition across checkpoint init,
freeze expiry and slot reuse, finite crystal geometry, committed drill HP and
Nanotech XP, continuous lava through creature colliders and Umbris phase shields
while frozen. Host release tests used unoptimized code generation for turnaround;
the Windows installer performs its normal optimized release checks.

264 launcher/tool tests passed, including four complete gadget pages, five
language catalogs, numeric/color validation, idempotent profile migration,
capability 96 when a gadget is enabled, local export integrity and updater
rollback. All 2,609 preceding public profile values match v132; 27 neutral gadget
settings were appended. The saved randomizer generator, WEVO save-format source
and custom icon are byte-identical to v132. No GitHub operations were performed.

The final artifact includes a fresh complete source inventory; ZIP CRC and every
source digest are checked. Windows rc-engine linking, the actual Tk GUI paint,
gadget joint layout, positional motor sound and GPU/in-game appearance were not
available on this host. The kit's Windows installer builds and verifies both the
runtime and GUI before transactionally replacing the playable files. Use
LOCAL_TEST_V133.md for the required in-game checks and the conditional tip-joint
and sound-asset details. No original game data or executable is included here.
