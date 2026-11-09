# v142 — return HUD ownership to the recalled weapon

Weapon Evolution 0.5 / ReRAC 0.3.8-alpha local test, runtime `0.1.6.142`,
capability 97, existing launcher 0.30.2 retained. Includes the v141 explosion
damage/XP attribution fix. No profile values, save formats or seed changes.

## Cause and correction

Accepted wrench XP starts a transient eight-second HUD feedback lease. The
renderer intentionally gives this lease precedence over the normal weapon
name/XP panel. The lease previously observed only XP serials, time, death and
level caps; it never observed a completed hand-item change. Native Circle/R2
recall can equip the saved weapon without a Quick Select request, leaving the
wrench feedback in front of the correct native ammo panel until its timer ended.

The lease now observes actual hand models after consuming that frame's XP
event. A completed change to another hand item retires both the timer and its
fade tail. This handles native recall, either Quick Select ring, D-pad bindings
and pause-menu equip through the same state, without button-specific guesses.
The renderer then uses its existing native name/XP/ammo lifecycle and geometry.
Neither native ammo requests nor font, bar dimensions, layering or colors change.

Empty holster/draw gaps are not equipment selections. A fresh delayed thrown-
wrench hit can still show feedback while the same weapon remains held; simply
holding another weapon must not globally disable accepted wrench-XP feedback.
An actual subsequent swap takes precedence, even if new XP arrives in that tick.
Death/reload reset the transient feedback state. Without a swap,
the existing eight-second game-time duration, fade, pause and V5/V8 caps remain.

## Verification

Regression cases cover recall to every weapon and the Freezer without a ring
request, empty hand gaps, returning to the wrench without fresh XP, subsequent
fresh accepted XP, natural fade tails, same-frame swap/XP priority and delayed
thrown-wrench hits. A shared HUD fixture runs weapon -> wrench -> saved weapon
using only held-item inputs and checks the native ammo panel/name/XP owner.
Existing timer, death/reload, pause and grade-cap tests remain in place.

The kit root `VERIFICATION.md` records host test results. These fixtures do not
claim a Windows or real-disc playthrough. The Windows installer tests/builds
and validates the exact runtime identity before replacing installed files.

## Play check

Choose an ammo weapon, hit an enemy with Square, then use Circle or R2 to recall
the saved weapon before eight seconds elapse. When its draw completes, its own
name and XP must accompany the native ammunition panel without opening Quick
Select. Repeat with ammo-less weapons, normal/locked camera and after a thrown
wrench hit. Natural wrench-feedback expiration must still fade smoothly.

Weapon Evolution is an independent mod by TomRus47, who develops the mod only
and is not associated with ReRAC or original-game development. Original-game
rights remain their owners' property. Thanks to the ReRAC developers and
contributors for the port which enables these mods; existing credits/licenses
remain applicable.
