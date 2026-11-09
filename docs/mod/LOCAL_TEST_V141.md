# v141 — native explosion attribution and XP

Local test for Weapon Evolution 0.5 / ReRAC 0.3.8-alpha. No publication action.
Runtime identity: `0.1.6.141`, capability 97; installed launcher 0.30.2 retained.
No balance values, approved defaults, seeds, save formats or icons are changed.

## Confirmed cause

The experimental Trespasser's private hit marker used bit 23 (`0x800000`).
That is already a native explosion flag. The shared `area_push_inner` correctly
copies native collision flags to `HitTemplate.w20`; its Bomb Glove, Mine Glove,
Visibomb and chicken-blast callers therefore also carried the drill marker.
The classifier checked this bit before the source class and accepted any
nonempty attacker pointer. It labelled these explosions as item 26, even when
experimental gadget combat was disabled.

This was not a Gold-only failure: normal and Gold explosions share this delivery
path. Misclassification selected the drill's continuous DPS conversion rather
than the configured weapon's per-hit damage, skipped the weapon XP recipient,
and could allow repeated explosion-frame events. Accepted damage could instead
credit Nanotech through the drill's reward path. It could also misclassify
unowned scenery/enemy area hits. Gold textures and grade/save migrations do not
participate in the failing decision.

## Fix

- Move the private drill tag to bit 26, outside native low-24-bit area flags.
- Require the real Trespasser source class and Ratchet ownership as well as the
  private tag. A copied flag alone never authorizes gadget damage or rewards.
- Retain existing per-instance explosion deduplication, accepted-HP accounting,
  source snapshots, health scaling, reaction handling and XP multipliers.

The direct native templates for Blaster, Pyrocitor/Lavagun, Tesla, Devastator,
R.Y.N.O., Doom, Drone Device, Walloper, Suck Cannon and Decoy use distinct metadata
and do not trigger this collision. Visibomb's experimental grenade ownership
override already precedes the classifier, as do companion/shotgun overrides.
Mine flame streams use the ordinary per-contact template, not native area_push.
Morph conversion rewards remain separate; chicken explosion damage does use the
affected area path. The fix covers all affected callers without changing those
unrelated templates or the native area flags used by crates and enemy filters.

## Regression coverage

The original all-weapon reward tests injected an already-labelled `Attack`.
They proved accepted-HP accounting after attribution, but missed the native
template-to-weapon boundary. The new fixture starts with `area_push`, including
its real flag copy, then consumes the queued hit, resolves damage, writes HP and
checks weapon/Nanotech XP. It covers all eight grades, Gold Off/On independently,
source deletion before consumption and repeated contacts from one explosion.
V5/V8 correctly stop weapon XP at their respective caps; accepted-health rewards
still follow the configured Nanotech rules.

Additional cases cover every native weapon identity, copied explosion/private
bits, real drill ownership, and unowned environmental explosions. Existing
tests cover independent second projectiles, stream timing, split amoeboid XP,
health/terminal accepted-loss accounting and captured drill damage.

The unmodified v140 classifier failed both initial regressions: a Bomb Glove
blast returned weapon 26 instead of 10, and a drill tag was accepted without a
valid Ratchet owner. The host validation log is recorded in `VERIFICATION.md` at
the kit root. Host tests use hand-built fixtures, not a visual game playthrough;
Windows compilation and real-disc behavior are checked by the included build.

## Local play checks

1. Close the game/launcher, compile the kit and select the existing playable
   Weapon Evolution 0.5 folder containing `weapons.ini` and `runtime/rerac.exe`.
2. Check the Bomb Glove and ordinary Mine Glove below their level caps, including
   V6/V7 after Gold purchase. An enemy's actual HP loss must award the owning
   weapon XP once, using current INI damage and XP settings.
3. Repeat with normal guided Visibomb and exploding Morph-o-Ray chickens.
4. Confirm the same behavior with Gold behavior disabled; check ordinary firing,
   continuous weapons, Doom, R.Y.N.O. and the experimental drill.
5. Repeated frames of one blast must not repeat damage/XP; a distinct second
   explosion may immediately inflict its own accepted damage.

Weapon Evolution is an independent mod by TomRus47, who develops the mod only
and is not affiliated with ReRAC or original-game development. Original games,
characters, assets and trademarks remain their owners' property. Thanks to the
ReRAC developers and contributors for making these mods possible. Existing
licenses and credits remain applicable.
