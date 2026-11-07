# Combat and crate rendering local test — v124

ReRAC **0.3.8-alpha**, Weapon Evolution **0.1.6.124**, configuration capability
**93**. This is a local test, not a GitHub update or release. The installed
launcher and custom icon are retained. Existing INI values, presets, saves,
randomizer seed and reward placements are preserved. No Visibomb behavior or
balance setting is changed.

## Amoeboid XP conversion

The old amoeboid-only branch treated accepted native HP times 100 as a fixed
weapon-XP award, ignoring xp_per_damage. Attraction compensated with an inverse
XP rate, producing the same inflated reward for Taunter/Decoy. Ordinary enemies
already applied the configured HP-to-XP conversion.

Placed amoeboids now use that common path for accepted direct damage, streams,
explosions, swallowing, morphing and attraction. Keep actual HP loss, the effective
enemy health scale, the configured XP rate and the active reward-chain factor
separate. Apply each factor once. Overkill and repeated delivery of one attack
cannot multiply the accepted loss. Attraction retains its configured fraction
and once-per-weapon/per-life receipt; Taunter and Decoy still grant no Nanotech XP.

The stages retain **300/200/100 HP**, and split only when their stage reaches
zero. Split descendants retain their damage behavior but award **zero weapon
and zero Nanotech XP**, including special-action rewards. An independently
placed small/medium amoeboid is still eligible.

At health scale 1 and reward factor 1:

| Placed size | HP | XP at rate 0.12 | XP at rate 1 |
| --- | ---: | ---: | ---: |
| Small | 100 | 12 | 100 |
| Medium | 200 | 24 | 200 |
| Large | 300 | 36 | 300 |

These are total damaging-weapon awards for that placed life, not extra death
bonuses. A Decoy configured to award 25% of a full large amoeboid's remaining
HP gives 9 weapon XP at rate 0.12, once. More HP or a legitimate reward-chain
bonus increases the award proportionally; a zero rate disables it. Already
earned XP in an existing save is not removed or rewritten.

Regression coverage exercises all damaging weapon IDs, all placed sizes,
duplicate primitives/frames, overkill, health scaling, zero/custom rates, NG+
reward factors, wrench swing identity, both streams, Doom explosions, attraction,
swallowing/morphing and split descendants through actual delivery/resolver/HP
writes rather than only invoking an XP helper.

## Suck Cannon crate presentation

Crates retain their original class, contents and native pose. Their cannon fire
path no longer requests a creature fire/bounce animation. Animation is frozen
while captured/in flight; the original mode is restored on an interrupted pull.
Launch restores the original scale, clears held/hide flags and immediately
rebuilds matrix/bounds. Capture/flight uses full draw/update distances instead
of the stationary crate's short range; cancellation restores both native values.

The original crate still owns collision sweeps, homing, the shot's damage
snapshot and native drops. TNT uses one cannon-owned blast; steel retains its
damage factor. One stored crate can be launched only once. Tests cover every
supported crate type, a nonzero native animation sequence, hidden zero-scale
storage, immediate restored bounds, canceled capture, source damage snapshots
and single-hit explosive attribution. GPU appearance must be checked in game.

## Veldin beam-manager clock

Class 1471 is a shared visual manager, although it also appears in the legacy
donor catalog. The general enemy clock consequently ran it multiple times at
increased speed. Its first call published the beams; its next call interpreted
them as unrefreshed and released them before rendering.

Exclude this manager from the enemy-AI speed classification. Actual beam drones
continue to use their configured clock. The manager publishes its effects once
per native tick and still releases a beam when the owner stops aiming it. No
boss, story-projectile, targeting, damage or XP catalog is broadened by this fix.
Tests run the real manager at speed settings 0.5, 1, 1.25, 2 and 4, checking a
live finite ribbon, one phase step and next-tick cleanup. This addresses the
identified Veldin case, not a claim that every possible enemy effect has been
visually playtested.

## Build and test

Extract the full kit into a new folder. Close the game and mod launcher, then
run **01_COMPILE_AND_UPDATE.bat**. Select the existing playable mod folder which
contains weapons.ini, ReRAC_Weapon_Evolution.exe and runtime/rerac.exe. The kit
includes complete source; no previous source-patch installation is needed.

The existing Visual Studio/Rust recovery and Cargo cache are reused. Source
integrity, Rust tests and actual EXE identity are checked before transactionally
replacing the runtime and matching shaders. This test uses --runtime-only,
which retains the working GUI EXE and icon. A failed build leaves the existing
playable runtime installed. Read logs/update-error.log if the installer fails.

Run **02_OPEN_UPDATED_LAUNCHER.bat** to reopen the same installation. Test a
placed large amoeboid with low damage and record incremental XP, then kill its
descendants; repeat with attraction and streams. Test all enabled crate types
through capture/fire/collision, and Veldin's beam drones in NG+ at increased
speed. **03_CREATE_PLAYABLE_ZIP.bat** can package the updated playable runtime
with the current personal INI, excluding saves, personal presets and game data.

Headless verification passed **1,426 rc-game library tests** with locked offline
dependencies, no default features and host optimization disabled. The complete
**224 Python tests** passed, including runtime-only installation without GUI
replacement. Export checks retain all default INIs and the icon byte-for-byte;
the actual extracted kit passes source hashes and the real build preflight.

The preparation host runs headless Rust/Python tests. Native Windows linking
and GPU/gameplay verification are performed by the local build and player test;
this kit does not claim those were performed on the preparation host.

Weapon Evolution is an independent mod by **TomRus47**, not part of ReRAC or
original-game development. The author owns no original-game rights, assets or
trademarks. Thanks to the ReRAC developers whose work makes these changes
possible. Upstream licenses, contributor attribution and original-game rights
remain with their respective owners. No original game data or saves are included.
