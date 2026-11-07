# Dual Quick Select local test — v125

Weapon Evolution **0.1.6.125**, ReRAC **0.3.8-alpha**, configuration capability
**94**, launcher **0.29.7**. Local test only. No GitHub operations or release
publication. Includes the v124 amoeboid XP, Suck Cannon crate and Veldin beam
fixes described in [LOCAL_TEST_V124.md](LOCAL_TEST_V124.md).

## Install and test

1. Extract the complete ZIP into a new folder. Close the game and mod launcher.
2. Run `01_COMPILE_AND_UPDATE.bat`. Select your **existing playable mod folder**
   containing `weapons.ini` and `runtime/rerac.exe`. Do not select this source kit.
3. The script compiles the bundled full source and rebuilds/self-tests the Windows
   launcher using the existing custom icon. Installation occurs only after checks
   succeed; a failed build keeps the previous playable executables.
4. Run `02_OPEN_UPDATED_LAUNCHER.bat`. Existing INI balance, presets, seeds and
   saved games are preserved. The new pause option defaults to **Off**.
5. `03_CREATE_PLAYABLE_ZIP.bat` creates a shareable package after a successful
   update. It does not upload or publish it.

This kit contains source and build scripts, not precompiled Windows executables,
original game assets or extracted disc data. Use your existing ReRAC installation
and extracted game data.

## Two rings

Hold Triangle in normal Ratchet gameplay. Press R2 to change from ring 1/2 to
2/2; press it again to return. A held R2 changes the ring only once. The R2 page
indicator remains visible even when the second ring is empty. R2 is consumed by
the menu and does not fire a weapon while Triangle is held.

In the pause menu's Quick Select editor, R2 changes the ring being edited. Each
ring retains eight independent slots; assigning an item moves its old binding
rather than duplicating it across rings. The last ring is shared by both views
and remembered for the next opening. Save the game to persist it in that slot.
A newly acquired world weapon fills a free slot in ring one, then ring two;
acquisition never replaces a full ring.

The second ring starts empty when loading an old save. Native first-ring slots
are not moved. Existing D-pad equipment shortcuts, R1 weapon modes, Clank command
pages, native equipment restrictions and Triangle double-tap behavior remain.

## Optional freeze

Global Settings → Base settings → **Pause while Quick Select is open**:

```ini
[general]
pause_on_quick_select = false
```

On freezes gameplay, AI, damage/projectiles, world animation, water and star/RNG
updates while the Ratchet radial is open. Input sampling, menu selection, R2 page
switching, supported R1 mode switching and menu fades continue on the UI clock.
Releasing Triangle equips the selection and resumes after the closing fade.
Paused world ticks are discarded, with no catch-up burst on resume. No pause
screen or world screenshot is shown. Clank's command page and non-gameplay modes
keep their native behavior. The setting can also be changed through live config.

## Pokitaru route check

The original Pokitaru Gold Bolt source 140 already requires Heli-Pack,
Thruster-Pack **and** Swingshot together in seed reachability. The added seabed
source 179 remains a different check without an O2/Hydro requirement. An outdated
route-document sentence has been corrected and all eight gadget subsets are
covered. This audit does not change existing seeds or reroll placements.

## Save compatibility

WEVO **v10** adds nine bytes before the checksum: eight ring-two item IDs and the
active ring (0/1). Native `saveN.bin` and the first ring's layout are unchanged.
The content hash/checksum still protects slot association. Readers migrate
v1–v9 with empty ring two and active ring one. Older runtimes cannot read the new
companion version; back up a slot before testing if a downgrade may be needed.
Ring/editor updates are copied independently of combat progress and do not reset
combat epochs or award/revoke weapon or Nanotech XP.

Default Randomizer save location on Windows (a custom save root takes precedence):

```text
%APPDATA%\rerac\memcard\randomizer\
```

Back up the complete regional card folder, including native saves and all `.wev`
companions. Do not copy or rename an isolated companion from a different save.

## Checks to perform in game

- Assign distinct weapons to both rings; switch once with R2, hold it, release,
  reopen Triangle, reopen the editor, save/reload and load a different slot.
- With pause On, open the ring next to a moving enemy/projectile. Wait, select,
  switch rings/modes, close and verify continuous resumption without a jump.
- Repeat with pause Off, mirrored controls, an empty second ring and Clank's
  command page. Normal movement/fire outside the radial must still work.

The mod is independent. Its author does not own the original Ratchet & Clank
games or assets and is not involved in developing ReRAC or the original games.
Credit belongs to the respective rights holders and the ReRAC developers whose
runtime makes this mod possible; see [CREDITS.md](CREDITS.md).
