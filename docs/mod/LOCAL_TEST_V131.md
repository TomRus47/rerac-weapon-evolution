# Local test v131 — Hoven fallback for an empty extra-site selection

ReRAC **0.3.8-alpha**, Weapon Evolution **0.1.6.131**, configuration capability
**95**, launcher **0.29.9**. This standalone local kit retains all v130 changes:
Taunter R1 modes, localized pause-menu names/XP, PDA buy-all ammunition, grey
bolt packs and the six new-seed location switches. Nothing is published to GitHub.

## Compile and test

1. Extract this complete ZIP into a new folder and close the game and launcher.
2. Run **01_COMPILE_AND_UPDATE.bat**. Select the existing playable mod folder
   containing weapons.ini, ReRAC_Weapon_Evolution.exe and runtime/rerac.exe.
   Do not select the extracted source kit or the ReRAC source checkout.
3. The script rebuilds both runtime and launcher, retaining the custom icon,
   personal values, presets and saves. A failed build retains the old playable
   executables. Read logs/update-error.log if the operation fails.
4. Run **02_OPEN_UPDATED_LAUNCHER.bat** to open the same installation.
   **03_CREATE_PLAYABLE_ZIP.bat** packages the updated playable mod without game
   data, saves or personal presets. No previous patch installation is required.

## All six Off: Hoven remains enabled

When creating a **new Randomizer All seed**, if all six extra-location switches
are Off, **only Hoven's extra location remains enabled**. This is the visible
suspended platform reached by gliding. Its existing route requirement still
applies: Ratchet needs a suitable Clank pack for gliding.

The fallback retains **every weapon, gadget, coordinate and all 40 Gold Bolt
rewards**. No Gold Bolt reward is removed to make room for the Bomb Glove.
The other five extra sites remain disabled in this case. With any nonempty
selection, exactly that selection is used; Hoven is not added automatically.
The choice affects only new All seeds, not Weapons-only mode.

Launcher help explains the all-Off fallback in English, Italian, French,
German and Spanish. The six switches may remain visually Off in the profile;
the saved seed records the effective Hoven-only mask used for generation.
Existing profile values and defaults are unchanged.

## Existing saves

Loading a save does not reroll, move rewards or force new sites into its layout.
The saved location mask remains authoritative, including a zero-site seed made
with the earlier v130 test. Its original reward pool remains readable for
compatibility; the old Gold Bolt omission is no longer used for **new** seeds.
NG+ continuation keeps the source game's saved layout and location choice.

The native save and WEVO v11 formats are unchanged from v130. Taunter mode,
both Quick Select rings, XP and collection receipts remain intact. An empty
new-seed selection is normalized only at generation, never during save decoding.

For the remaining v130 controls and implementation notes, see LOCAL_TEST_V130.md.
Its all-Off Gold Bolt replacement policy is superseded by this Hoven fallback.
Veldin's electrical-beam fix remains included. The Blarg Station immobility
report still needs in-game confirmation; no unrelated AI workaround is applied.

## Verification scope

The fallback test compares all-Off generation with explicit Hoven-only generation,
checks the full reward multiset and all 40 Gold Bolts, verifies route-model
solvability and confirms that no other extra site is present. A separate fixture
round-trips an earlier v130 zero-site save without changing its mask or mapping.
All 63 nonempty masks retain their original meaning. Automated source, launcher
translation and archive integrity checks complement the local Windows build.
Route-model checks do not prove every physical path. This host does not run the
Windows game or its GPU rendering; the included installer checks the Windows
runtime build and frozen launcher before replacing installed executables.

Weapon Evolution is an independent mod by **TomRus47**, who is not associated
with development of ReRAC or the original Ratchet & Clank games. The author
claims no original-game rights, assets or trademarks. Thanks to the ReRAC
team, whose continuing work on the port makes mods like this possible.
Original-game rights and upstream licenses/credits remain with their owners.
Original game data is not included.
