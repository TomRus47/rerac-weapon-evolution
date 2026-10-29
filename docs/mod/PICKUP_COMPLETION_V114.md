# v114 local test — pickup scene completion

Runtime **ReRAC 0.3.8-alpha**, Weapon Evolution **0.1.6.114**, configuration
capability **90**, existing launcher **0.29.3**. All v113 XP fixes remain.
The upstream audit records the historical v112 merge; the current source index
tracks this local patch.
No GitHub operations are performed by this kit.

Weapon Evolution is an independent mod by TomRus47. The author is not associated
with ReRAC development and owns no original-game rights or assets. Thanks to the
ReRAC developers for their work and for the runtime that makes this mod possible.
See [credits](CREDITS.md) and the upstream licenses.

## Reproduced Gold Bolt failure

The randomized Gold Bolt cutaway uses a newly created cosmetic actor. Dynamic
actors start with update distance zero. Once the cutaway camera leaves that point,
or the bolt becomes hidden, the normal active-list builder can stop scheduling
that actor. Its animation and final timer stop too, leaving Ratchet in the hold-up
animation and scripted camera. The earlier test called the class update directly
and manually set the ending sequence, so it missed this scheduler boundary.

New regression tests reproduce the failure using the real active-list builder,
an actual four-key bolt animation, an offset camera and visibility held at zero.
Neither animation-completion flag nor ending sequence is forced in that test.

## Correction

Only the tagged cosmetic pickup actor receives:

- Always-active update distance, an explicit Gold Bolt update function and a
  normal draw distance. Its hidden tail continues without renderer visibility.
- Cleared update/animation suppression and collision/targetability disabled.
  Class tables and ordinary story/pickup actors are not globally rewritten.
- A resource check before taking control: both bolt sequences must have frames.
  Missing clips use the existing localized banner/jingle fallback.
- One presentation owner. Another request cannot stack a second scripted hold.
- Normal native animation-wrap completion and tail timing, plus a ten-second
  safety budget in reference ticks for an unexpectedly stalled animation.

Normal completion and safety cleanup release the letterbox/script camera, reveal
the hand item, restore idle/control for a living hero and delete the cosmetic
actor. Death cleanup preserves the native death state instead of forcing idle or
reviving Ratchet, and does not introduce a save at a dead pose. The reward has already been granted: presentation never writes
another native Gold Bolt flag or grants the randomized source again.

## Other added acquisition scenes

Native item scenes and weapon demos use a different player: `ShipMode` owns a
scene-frame counter, which advances even while the ordinary world counter is
frozen. Completion is based on the streamed scene's end tick, not a moby's
visibility or a hero animation-wrap flag.

Tests exercise multiple scene lengths and chunk boundaries with the world clock
held constant, and verify exactly one completion and actor cleanup. Missing demo
state or an empty scene now takes that same completion path, allowing the engine's
existing handler to restore the saved camera, gameplay mode, hero/HUD visibility
and held soundtrack. It cannot silently wait forever for absent resources.
The existing one-shot acquisition jingle and music-resume path remain in place.
The item-movie player has its own end-of-stream/skip handling; its behavior is
unchanged. Underwater pickups retain their banner/jingle-only policy.

## Compatibility and installation

No INI values, presets, saves, seed layouts or XP balances are changed. Native
saves and WEVO v9 companions retain their formats. Existing progression, including
already collected rewards, remains valid. This kit does not duplicate a reward to
repair its presentation.

Extract into a new writable folder, close the game, run
`01_COMPILE_AND_UPDATE.bat` and select the **existing playable mod folder** with
its launcher EXE, `weapons.ini` and `runtime/rerac.exe`. The installer builds and
tests these bundled sources before replacing the runtime and matching shaders,
with backups and rollback. Python, Rust/Cargo and installed Visual Studio C++
x64 tools/Windows SDK are needed. It initializes the installed MSVC environment.
The working launcher EXE, icon and player files are retained; no GUI rebuild.

`02_OPEN_UPDATED_LAUNCHER.bat` opens that same installation.
`03_CREATE_PLAYABLE_ZIP.bat` packages the updated runtime with its existing GUI
and current INI. No personal saves, presets or disc assets are exported.

In-game checks on Windows: collect a randomized Gold Bolt on foot, then collect
another reward; verify one animation, disappearance, hand item, HUD, camera and
movement. Check a weapon and a gadget with a native acquisition scene, including
that the jingle ends and the previous music resumes. A previously stuck in-memory
session must be closed before installing; the updated runtime handles future
presentations. See [verification](VERIFICATION.md) for automated evidence and
host limitations. No Windows/GPU playtest is claimed by the preparation host.
