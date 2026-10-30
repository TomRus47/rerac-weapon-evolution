# Combat and pickup local test — v117

Base: ReRAC 0.3.8-alpha. Runtime patch: 0.1.6.117; launcher: 0.29.6;
configuration capability: 93. This is a local test, with no GitHub operations.
Existing seeds, native/companion saves, profiles, balance and custom icons are retained.

Weapon Evolution is an independent mod by TomRus47. Its author is not involved
in developing ReRAC and owns no original-game rights or assets. Thanks to the
ReRAC developers for the work that makes these modifications possible. Original
rights, upstream licenses and credits remain with their respective owners.

## Changes

- Gold Bolt/reward presentation cannot interrupt a grind, rail jump, rail attack
  or rail hurt state. Native no-cutaway location flags also survive randomized
  reward anchors. The reward, localized notification and sound still occur.
- Retained enemy actors are rearmed for Morph-o-Ray only after native code has
  restored a living, targetable life and consumed its morph marker. A callback
  repeated for the previous life cannot award another transformation or XP.
- Kalebo III's camera-owning weapon-test controllers flatten locked combat aim.
  The scripted camera itself remains in charge. Other cameras and arena
  controllers without a camera spline retain their previous behavior.
- Quartu's Bolt Grabber actor becomes a physical randomized source. Its native
  model and native collection scene no longer precede an unrelated reward.
  The actual reward selects the replacement model and collection feedback.
- `general.enemy_burst_tracking` is an Off/On encounter option, initially Off.
  Eligible burst shooters can re-aim new direct shots at their current target.
  Decoys remain valid targets; projectiles already in flight never acquire this
  behavior. Scripted destruction shots, boss attack patterns and Hoven's mounted
  turret mission retain native aim. The setting supports live profile edits.
- With Randomizer All, actual early O2 ownership and pending Orxon Clank checks,
  only the vertical shortcut ending closest to the native Magneboots pedestal
  is unlocked and initially lowered. Clank's section controller, other gates,
  puzzle state, mission flags and item ownership are left intact. Ambiguous or
  malformed platform geometry is rejected instead of opening arbitrary lifts.
- Each accepted wrench XP award refreshes an eight-second name/XP feedback lease.
  The ordinary fade envelope is used. No auto-feedback is generated at the
  normal-game V5 cap or NG+ V8 cap; V5 in NG+ can still earn XP and show feedback.
  Triangle-only display can also show this hit feedback; the per-grade HUD
  master switch, swimming, death and scripted HUD suppression still apply.
  A melee hit can show the wrench without changing the equipped weapon or its
  ammo counter. The lease is transient, not a new save field.

## Implementation boundaries

`weapon_evo_golden::pickup_cutaway_allowed` is shared by native Gold Bolts,
randomizer feedback and deferred scenes. Presentation restrictions do not
change inventory grants or permanently equip an item during a grind.

`World::renew_retained_enemy` resets only consumed per-life reward/hit identity.
It does not reset HP, change native respawn timing or bypass targetability.
Normal CreateMoby allocation continues to handle reused dynamic slots.

`kalebo_arena::fixed_weapon_camera` identifies the scoped controllers;
`Hero::fixed_combat_aim` is recomputed every gameplay tick. The gun-direction
helper uses horizontal body yaw only during locked combat in those rooms.

`weapon_evo_burst` changes launch vectors in the direct-shot constructors used
by the hopper gunner, drop trooper, Kerwan trooper, shared Orxon/Quartu drones, Hoven drone and
shared Eudora/Hoven gun shot. It retains native speed and auxiliary vector data.
Native enemies that already track their target continue to do so.

`weapon_evo_orxon_shortcut` identifies the shortcut from native actor and spline
geometry. Only its local appear/activation fields and starting endpoint change;
no persistent global completion latch is manufactured.

`Evolution::wrench_xp_feedback_serial` increments after a positive accepted
weapon award, including fixed per-life rewards. Nanotech awards alone, rejected
hits and disabled/max-grade wrench tracks cannot trigger it. `hud::wrench_xp`
measures the lease in gameplay ticks, so pause does not consume its duration.
The renderer keeps the existing font and XP primitive batching.

## Windows test procedure

Extract the complete kit to a new folder, close the game and launcher, and run
`01_COMPILE_AND_UPDATE.bat`. Choose the EXISTING playable folder containing
`weapons.ini`, the launcher EXE and `runtime/rerac.exe`. Do not choose this kit's
source folder. The script initializes installed Visual Studio x64 tools, checks
bundled source integrity, runs Rust tests, compiles the runtime and runs the
standalone launcher GUI self-test before replacing installed files. Failures
retain the previous working installation; replacements have rollback backups.

Use `02_OPEN_UPDATED_LAUNCHER.bat` to reopen the exact updated installation and
`03_CREATE_PLAYABLE_ZIP.bat` to package its current settings and executable.
No new save is needed for these fixes; existing randomizer layouts are preserved.

Verify in game:

1. Collect Kalebo's grind-rail reward without losing rail control.
2. Transform several successive lives of a Kalebo test robot and enemies emitted
   by machines. Confirm one transformation/XP award per life.
3. Strafe in the fixed-camera test rooms, then outside them: room aim is horizontal
   and ordinary camera aim remains available outside.
4. Collect Quartu's replaced Bolt Grabber check: model and feedback match its reward.
5. Compare a burst shooter with the new option Off and On; move during its burst,
   then test a decoy. Earlier shots must retain their original trajectory.
6. Enter Orxon with an early O2 Mask and pending Clank checks: Clank begins first,
   the shortcut is available below, and the remaining puzzles still require play.
7. Earn wrench XP, wait eight seconds, hit again during the interval, pause, swim,
   die and load a slot. Check the fade and normal/NG+ grade caps separately.

Automated verification and host limitations are recorded in VERIFICATION.md.


## Installer correction 117.1

The first v117 export retained v116/92 literals in the pre-Cargo source API
check. It stopped before cleaning/building or installing any playable files.
The verifier now derives the expected patch/capability from the bundled
`mod/development.json` and still rejects mismatched engine identity markers
or missing gameplay/HUD interfaces. The exporter runs that same preflight
before producing a ZIP. Five regression tests cover the current source,
version changes, stale markers, missing interfaces and export refusal.
Gameplay, launcher implementation, INI values and save formats are unchanged.
Extract the replacement kit into a fresh folder and run its compile entry
point against the same existing playable installation. No v116 installation
or manual runtime replacement is required.


## Automatic Rust recovery — installer 117.2

Before selecting or cleaning a Cargo cache, the installer resolves real compiler
and Cargo paths with Rustup, executes both, and compiles a temporary metadata
probe. A Rustup manifest claiming that a missing compiler is installed is not
accepted as proof of a usable toolchain.

If installed Rust fails, the installer first checks a previously verified
kit-local toolchain. Otherwise it installs `stable-x86_64-pc-windows-msvc` with
the minimal profile under a new `build-tools/rustup-<id>` inside this kit.
This download uses the existing Rustup executable and requires internet.
The old Rust installation and Cargo dependency cache are retained. No global
uninstall, permissions change, administrator action or security exclusion is
performed. The selected RUSTUP_HOME/RUSTUP_TOOLCHAIN/RUSTC/PATH apply only to
this installer process and its children. The cache marker is written only
after actual compiler and Cargo probes succeed.

A missing, partially installed or unusable cached toolchain is not reused.
If Windows prevents the separate compiler from executing too, the installer
stops before Cargo cache cleaning and leaves the playable installation intact.
The exact process error is retained in the normal update diagnostic log.
Gameplay, launcher, profile values, seeds and save formats remain v117.


## Reset download recovery — installer 117.3

A network failure while installing isolated Rust now triggers one alternate
Rustup transport attempt. The configured/default HTTPS backend is tried first;
`RUSTUP_USE_CURL=1` selects Rustup's supported but deprecated cURL backend for
the fallback. If cURL was already explicitly selected, the fallback is the
default backend instead. Only child-process variables change. Existing proxy
configuration and distribution-server selection are retained. Certificate
validation and Rustup's checksum validation are not bypassed.

Both attempts stream their real progress into the existing diagnostic log.
Only network/download errors trigger this fallback: an access-denied or install
failure is not retried as if it were a network problem. Retries are bounded and
an unusable new compiler is still rejected by the compile probe. The fallback
may be unavailable in Rustup builds that omit cURL; that actual error is reported.
If both transports fail, check access to the reported Rust distribution endpoint
in a browser or retry this same kit on another connection. A network policy,
local filter or service interruption can still prevent the download; this patch
does not change network/security policies or install another distribution mirror.

References: https://rust-lang.github.io/rustup/network-proxies.html and
https://rust-lang.github.io/rustup/dev-guide/tips-and-tricks.html.
Gameplay, profile values, launcher and save formats remain unchanged.
