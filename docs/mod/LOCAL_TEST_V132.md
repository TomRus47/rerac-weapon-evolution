# Local test v132 — shop acquisition videos and NPC purchase prices

ReRAC **0.3.8-alpha**, Weapon Evolution **0.1.6.132**, configuration capability
**95**, launcher **0.29.9**. The launcher interface and controls are unchanged.
This complete standalone source kit retains every v131 change, including Hoven's
all-Off location fallback. No GitHub operation or release is performed.

## Compile and test

Extract this ZIP into a new folder. Close the game and mod launcher, then run
**01_COMPILE_AND_UPDATE.bat**. Select the existing playable mod folder containing
weapons.ini, ReRAC_Weapon_Evolution.exe and runtime/rerac.exe. Do not select this
source kit or the old ReRAC checkout. No earlier patch needs to be installed first.

The installer verifies the included source and builds the modified runtime and
matching launcher. Existing profile values, presets, saves, randomizer seed,
reward placements and the custom icon are retained. The GUI remains version
0.29.9 because its code and controls did not change. A failed build keeps the
previous playable executables. Read logs/update-error.log if the operation fails.

**02_OPEN_UPDATED_LAUNCHER.bat** opens that same installation.
**03_CREATE_PLAYABLE_ZIP.bat** packages its current launcher, runtime, shaders and
configuration without disc data, saves or personal presets.

## Randomized shop rewards: show acquisition media after closing

Some gadgets have explanatory movies, including the Metal Detector. Randomized
shop purchases already queued their actual reward in the session, but closing
shop animations run world ticks too. The engine drained the queue during those
vendor frames and started the movie before completing shop cleanup. Returning
from or skipping the movie switched to gameplay with a still-active vendor.

The receipt queue now waits for normal gameplay in both menu and world state,
with a living hero out of the vendor's held state. The shop completes its normal
exit first: its native/PDA actor, camera, audio and UI are released before any
acquisition presentation starts. One queued receipt is consumed per eligible
tick, before requesting its movie/cutaway, so skipping or finishing cannot replay
it. A second purchased gadget waits until control returns from the first movie.
Inventory ownership, payment and the source's single-claim record still update
at purchase time, rather than waiting for the presentation.

This also serializes ordinary randomized acquisition scenes and notices from
shop purchases. World pickups keep their existing presentation policy. The
queue is session-only and is not a new save field. A load or hero reinitialization
retains the established session-reset behavior.

Test a randomized Gadgetron offer yielding Metal Detector. Buy it, stay in the
shop, then close normally. The explanatory movie should begin only afterwards.
Try both watching it and skipping it. Gameplay must resume with the shop closed,
Ratchet controllable, and the gadget permanently owned. If another acquisition
movie is purchased during the same visit, presentations must run separately.

## NPC prices follow the original payment record

Randomized reward IDs and source/location IDs do not identify NPC price records.
For example, Novalis's plumber source is 66, but its authored payment pseudo-item
is 37. Looking up price 66 returned zero even though the actual NPC charge still
used record 37. Randomized prompt text now resolves the original payment node.
The original free, trade, flag and Gold Bolt conditions remain separate.

The same scoped price helper serves the affordability test and actual bolt debit.
Every NPC retains its native price regardless of the reward assigned to its
source. Price tables are not rewritten and a free offer cannot inherit the price
of an unrelated offer. The Novalis plumber therefore displays and charges the
original **500 bolts**, including when its randomized reward is the O2 Mask.

### Requested exception: Rilgar's R.Y.N.O. salesman

In **Randomizer All only**, the original R.Y.N.O. purchase from the specific
salesman on Rilgar costs **15,000 standard bolts**, whatever reward that source
contains. Its generated localized prompt, affordability condition and actual
charge agree. The old amount embedded in native dialogue text is not retained
in the generated purchase prompt. The original price remains in normal games;
other NPCs, other planets and other payment records are unaffected. This does
not create an additional sale in Weapons-only mode.

Check both affordability boundaries: 499/500 bolts for the plumber and
14,999/15,000 bolts for the randomized R.Y.N.O. offer. A denied purchase must not
debit bolts. The successful purchase must subtract precisely the displayed cost.

## Verification and compatibility

Native Rust regression fixtures exercise actual randomized item grants, retained
vendor receipts, serial presentation after movies, all NPC payment records,
Novalis/Rilgar affordability boundaries, the actual debit and localized prompts.
The full test scopes and host limitations are recorded in VERIFICATION.md.

Balance INIs, launcher sources/icon, saves, WEVO v11 format, location catalogue,
seed generation and placed rewards are unchanged. No existing purchase is
refunded, recharged or made repeatable. Previously collected rewards remain
collected. No diagnostic audio recording or default tracing is introduced.

The preparation host does not run the Windows game or a GPU visual test. The
included installer validates the Windows build and frozen GUI before replacing
installed executables; the above movie/shop checks still require playtesting.

Weapon Evolution is an independent mod by **TomRus47**, who is not associated
with development of ReRAC or the original Ratchet & Clank games. The author
claims no original-game rights, assets or trademarks. Thanks to the ReRAC team,
whose continuing work on the port makes these mods possible. Original-game
rights and upstream licenses/credits remain with their respective owners.
No original game data is included.
