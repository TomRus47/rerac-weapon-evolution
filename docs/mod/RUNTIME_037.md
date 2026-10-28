# ReRAC 0.3.7-alpha integration

Weapon Evolution 0.4 draft candidate: gameplay **0.1.6.110**, capability **90**,
launcher **0.29.3**. The public 0.3 release/tag remain unchanged. See the
[complete draft notes](../releases/v0.4-rerac-0.3.7-alpha.md) for the feature
history and [verification](VERIFICATION.md) for build evidence and limitations.

Weapon Evolution is an independent mod by TomRus47, who develops the mod only,
is not a ReRAC developer and owns no rights to the original games or assets.
Thanks to the ReRAC developers for their ongoing work and for providing the
runtime that enables this project. Upstream licenses and credits apply.

## Exact source boundaries

The mod base is main commit `e2269eb4afd5574d3f70c9d19a612d52c771102e`.
The complete official upstream delta is integrated from 0.3.6-alpha commit
`89c87e98f9d8296187df2a6310c67b32144090d3` to 0.3.7-alpha commit
`f2dfbb84d7025d475ce8e39f9ab4ae345ab35133`.

The [official release](https://github.com/re-rac/rerac/releases/tag/v0.3.7-alpha)
and root changelog describe its fixes. `mod/upstream_037.json` records all 58
upstream paths, original and new blob IDs, and the integrated result. Existing
mod integrations are merged, not replaced by vanilla versions. The complete new
forum-triage document is restored because the old export omitted its base;
the official changelog section is added without discarding mod history.

## Merge decisions

- **HUD compositor:** retain the native alpha compositing, resize bindings and
  batch layout alongside weapon/health XP, name vignettes and configurable bar
  stacking. The updated WGSL files must travel with the matching runtime EXE.
- **HUD ownership:** `weapon_reset_pending` tracks only the reset hold that lacks
  a vendor exit after death/page reload. Recovery must not clear the separate
  hoverboard race hold. Ammo-less XP follows that gate; outgoing real panels can
  still fade naturally. Regression tests cover both ammo weapons and the wrench.
- **Glove aim:** pass the official hand-yaw reference through the mod's
  configured range-aware search so per-grade reach is preserved.
- **Water:** normal bombs recognize both terrain and moby water. Alternate
  contact explosions for clusters and Visibomb grenades remain explicit.
- **Platforms:** native ordnance retains upstream behavior. Fire mines return
  before native seeking, so bind their carrier on landing and apply the shared
  `triggers::carried` transform before light/sound/flame work. Body and cross yaw
  move together; captured DPS, palette, size, active limit and lifetime persist.
  A lost carrier retires the mine and releases light/sound ownership. The new
  test exercises translation, rotation, snapshots and cleanup in a moby world.
- **Services:** retain both mod stun tests and the native texture-scroll helper.
- **Diagnostics:** the new native inspect HUD is opt-in with `RC_DEBUG_HUD`.
  The launcher does not enable it. Removed temporary mod audio probes remain
  absent; runtime identity reports audio diagnostics disabled.

Other included fixes cover Swingshot release, widescreen aim, terrain UV scale,
Oltanis/Quartu scripts, Gemlik effects, Novalis flowers, PDA background, teleport
arrival, Visibomb scanlines and cinematic rendering. Existing damage/XP, saved
controls, randomizers, progression and weapon experiments remain integrated.

## Configuration and save compatibility

No new settings or capability requirements are introduced. The newly supplied
`GBtW_All_Custom(2).ini` is authoritative: all 2,592 values are preserved exactly.
Only English compatibility and obsolete boss-rule comments are corrected.
`mod/default-profile.json` records the supplied hash, public hash and complete
value fingerprint. Public default/example and optional embedded profile agree.
The installer/build does not overwrite personal weapons.ini or presets.

WEVO v9 remains current, with legacy decoding retained. There is no companion
migration, seed regeneration or save-directory change. Randomizer generation is
still experimental and can produce blocked runs; see the draft's reporting
instructions for the Randomizer-specific save and complete companion directory.

## Build and release boundaries

`01_BUILD_GAME.bat` builds the editable runtime from this complete workspace;
`BUILD_FINAL_WINDOWS_EXE.bat` creates the frozen launcher/player package. No
previous patch ZIP is required. Keep runtime/rerac.exe and its shader directory
together. The existing 0.29.3 launcher and custom icon remain compatible.

The draft workflow tests the exact pushed source on Windows, builds runtime and
launcher, verifies their identities and approved profile, and attaches player
and exact-commit source ZIPs with checksums. `mod/draft.json` requires draft=true
and publish=false. The published `mod/release.json` is historical and unchanged;
ordinary source pushes cannot publish a release. Neither game data nor saves,
presets, private paths or executables are committed to the repository.

Passing tests or compilation does not validate every disc-specific script,
physical route, collision or GPU output. The author's attached native package
contains the v110 marker and shader bytes matching these sources; CI rebuilds
the release assets from the recorded commit rather than inferring provenance
from that binary alone.
