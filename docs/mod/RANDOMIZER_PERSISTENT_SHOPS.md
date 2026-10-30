Weapon Evolution v123 - Persistent Randomizer Gadgetron offers
ReRAC 0.3.8-alpha. Local test only. No GitHub operations.

Behavior
--------
In Randomizer All, entering a planet registers that planet's native Gadgetron
sale in the persistent vendor stock. Its randomized reward is now available
from every normal Gadgetron vendor until purchased, instead of being hidden
whenever the player leaves its original planet.

Example: visit Novalis without buying the replacement for the Pyrocitor;
visit Kerwan and the vendor sells both that retained Novalis offer and the
new replacement for the Blaster. The price, reward name and model continue
to refer to the same source/reward pair. Buying it claims that source once
and removes the offer from all vendors. Stale transactions cannot charge or
grant it again. Obtaining the original source weapon elsewhere does not buy
that separate randomized reward.

This uses the existing saved stock (native save chunk 12) and existing
randomizer claims. No new INI setting, save format or seed regeneration is
needed. Stock is retained across native/companion save reload and NG+.
Existing registered offers become available immediately on older v122 saves.

Finding a weapon in the world still unlocks its ammo, without adding a shop
source or exposing unvisited planets' sales. Unlocking planet coordinates
alone is not a visit. NPC rewards and physical pickups are not Gadgetron
sales. RYNO's NPC purchase stays separate. The PDA remains ammo-only.
Normal mode, Weapons-only mode, pricing, balance and default profiles retain
their existing behavior.

Race completion fix (Kalebo III and Rilgar)
------------------------------------------
v122 corrected the host's reward offer, but the hero's race-finish code still
checked whether the native prize was owned. An early Hologuise could prevent
the Kalebo finish command from reaching the host at all. An early native
Rilgar prize flag had the same potential failure before the host could run.

A transient board-world snapshot now carries the original race-source claim
in Randomizer All. First place sends the proper host completion command while
that check remains unclaimed, even when the native prize was found elsewhere.
Previously claimed prizes keep the native repeat-race flow. Losing never
awards a prize. Actual gadget ownership, boosting, scores, records and native
mode are unchanged. The snapshot field is not serialized.

After updating, win Kalebo III again and complete the host's prize dialogue.
In the supplied seed the reward is the Persuader. The source Hologuise movie
is suppressed for random rewards; the actual reward uses the shared receipt
feedback. A prize with no native acquisition scene does not invent one.
Earlier race-finish counts do not prove a win, so this patch does not silently
grant unclaimed race checks from those counters or edit the player's save.

Installer shutdown fix
----------------------
Restore stdout/stderr before closing the build log. This removes the false
"Exception ignored while flushing sys.stdout / Operation failed" message at
Python shutdown after an otherwise successful update. Actual failures still
return a nonzero exit code and retain the diagnostic log.

Install
-------
1. Extract into a NEW folder. Close the game and the mod launcher.
2. Run 01_COMPILE_AND_UPDATE.bat.
3. EXISTING PLAYABLE MOD: choose the folder with your working launcher EXE,
   weapons.ini and runtime/rerac.exe.
4. PREVIOUS TEST KIT: choose the extracted v122 folder that you already built,
   or its source subfolder. Do not choose the playable folder or the old
   C:\ReRAC-Mod\rerac checkout. v122 remains untouched; this kit privately
   copies its fully inventoried source and retains all its preceding fixes.
5. Rust/Visual Studio recovery tools and dependency cache are reused. Tests
   and executable identity checks precede transactional runtime installation.
   The current launcher EXE/icon is reused; it does not need rebuilding.
6. Run 02_OPEN_UPDATED_LAUNCHER.bat or open your usual launcher. Load the same
   save and open a normal Gadgetron vendor to see the retained offers.
7. Optional: 03_CREATE_PLAYABLE_ZIP.bat packages the updated playable mod.

Only vendor availability, race-finish reward routing, regression tests, version/integrity metadata
and this document change in gameplay source. INI values, profiles, saves,
seed, mappings, NPC fixes and unrelated gameplay code are preserved.
A failed preparation/build leaves the installed playable runtime in place.
Logs: logs/v123-*.log and logs/update-error.log in this kit.

Verification
------------
Regression coverage exercises registration through actual planet-entry code
for all eleven Gadgetron sources, retained offers on a different planet,
world-found weapons versus shop sources, ammo availability, cross-planet
purchase at the original source price, single-claim protection, native save
and companion reload, NG+ persistence, and the PDA's ammo-only policy.
Race tests exercise the actual three-lap hero finish, its command dispatch
to each host, early native ownership, prior claims, losses, and the Kalebo
receipt followed by the randomized Persuader grant exactly once.
Installer tests verify full-source preservation, input/prepared integrity,
idempotent edits, path guards and console-stream restoration on success
and failure. Host verification passed 30 vendor tests, 3 complete race-finish
tests and 9 retained v122 NPC/reward tests, plus 7 updater/logging tests.
Windows linking and visual in-game behavior require this local test build.

Credits and rights
------------------
Weapon Evolution is an independent mod. Its author is not associated with or
part of ReRAC development and does not own the original Ratchet & Clank games,
characters, assets or trademarks. Thanks and credit to the ReRAC developers
whose runtime makes these modifications possible. No original game assets
or personal saves are included. Upstream retains its license and attribution.
