# v111 local test — reward presentation and combat

Runtime: ReRAC **0.3.7-alpha**, Weapon Evolution **0.1.6.111**, configuration
capability **90**, existing launcher **0.29.3**. This is a local test kit. No
repository, tag, draft release or public download is changed by its installer.

Weapon Evolution is an independent mod by TomRus47. The mod author is not part
of ReRAC development and owns no original-game rights or assets. Thanks to the
ReRAC developers for the runtime and continuing work that make this mod possible.
See [credits](CREDITS.md) and the upstream licenses.

## Changes

### Moving Comet-Strike

Both weapon-use checks and the direct crouch transition require the single
wrench to be attached, ready, of the wrench class and in its in-hand state.
Rejecting a second throw only in projectile creation was too late: entering the
throw state had already started its animation, stationary physics and voice.
Movement after the configured throw remains available. The same instance must
return before another melee/throw attack starts; no extra wrench is spawned.

### Localized reward and NPC text

Pickup notices use a generic localized "You found: <reward>" in English,
French, German, Spanish and Italian, preserving the disc's font-encoded name.
There is no weapon/gadget classification in that sentence. Missing names have
an item-neutral fallback.

NPC offers resolve the actual randomized source from their condition/table.
In particular Pokitaru's Raritanium flag is not an item index: its exchange is
bound to the Persuader source even when the reward is a planet coordinate.
Named source rewards are replaced in supported-language prompts and talker
subtitles. An unnamed/oversized offer gets a compact localized reward/price
prompt. Native conditions, costs and grants are unchanged. Orxon's separate
Nanotech sale already has a randomized reward prompt and keeps that path.
Original recorded voice performances are not replaced or regenerated.

### Acquisition scenes

The global vendor demo array is queried only for its fifteen weapon entries.
A zero-initialized gadget entry must not select demo zero (Bomb Glove).
Confirmed item-specific streamed presentations are staged on the loader worker:

| Reward | Native level | Scene |
| --- | --- | --- |
| Suck Cannon | Eudora | 2 |
| Morph-o-Ray | Oltanis | 6 |
| Hydrodisplacer | Blarg Station | 2 |
| Trespasser | Aridia | 4 |
| Magneboots | Orxon | 2 |
| Bolt Grabber | Quartu | 5 |

The existing Hologuise, Metal Detector and Sonic Summoner movie mappings remain.
Camera and actor tracks are translated together to the collection position,
using Ratchet or the native Clank actor as the player anchor.
Playback is isolated: foreign mission updates, scene-end item grants and travel
scripts do not run. The normal pickup player restores the previous camera/world
and soundtrack. Missing/corrupt scene resources fall back to a banner and one
jingle, never an unrelated weapon scene. World assets are staged before actor
creation, with no synchronous foreign-level loading during collection.

A randomized Gold Bolt instead uses the original in-world Gold Bolt cutaway:
Ratchet's hold-up animation, native bolt animation, camera, letterbox and class
sound. A tagged presentation actor cannot award another native bolt or location.
It releases control and deletes itself at completion. It does not request the
extra pickup jingle. Underwater collection keeps its no-cutscene policy.

### Boots across travel

Randomized Magneboots/Grindboots persist their feet selection immediately, rather
than waiting for a later gameplay tick before a save/travel. Missing worn models,
animations and joint lists are imported on the destination loader worker. Foot
pose scaling hides bare feet only for a ready/put-away real boot attachment,
not an empty or intermediate slot. Manual re-equipping is not required after
travelling to a planet without the native boots class.

### Amoeboid XP

Historical v111 behavior: [v113](AMOEBOID_XP_V113.md) corrects the Nanotech
conversion while retaining the fixed weapon budgets described below.

Placed small/medium/large amoeboids offer **100/200/300 base XP**, consumed through
confirmed HP loss. Their native HP and their children's HP are unchanged. The
explicit XP budget does not depend on the configurable XP-per-HP conversion or
health multiplier. Existing reward-chain bonuses still apply; zero XP-per-HP
still disables progression. Weapon XP and enabled Nanotech XP use the same
accepted amount, with the existing Taunter/Decoy exclusion for Nanotech.

Every child activated by splitting gets an actor-owned marker beyond native
pvars. All descendant generations provide zero direct-damage, suction, morph or
attraction XP. Damage, reactions, splitting and visual feedback still work.
The marker is not a slot-ID blacklist, so a new placed enemy in a reused slot
retains its own budget. Saves/seeds are not regenerated.

## Build and test

Extract the complete ZIP into a new folder. Close the game and run
`01_COMPILE_AND_UPDATE.bat`. Select the **existing playable mod folder** that
contains the launcher EXE, `weapons.ini` and `runtime/rerac.exe`. Do not select
this source kit. Python, Rust and Visual Studio C++ x64 tools/SDK are required;
the installer initializes the installed toolchain, tests and builds the bundled
sources, checks runtime identity, then updates the runtime and matching shaders.
The launcher/icon, personal INIs, presets and native/WEVO saves remain intact.
A failed installation rolls back its replacement set. Logs and backup paths
are printed. `02_OPEN_UPDATED_LAUNCHER.bat` reopens that exact installation;
`03_CREATE_PLAYABLE_ZIP.bat` packages it with the current INI and existing GUI.

Automated evidence and host limits are in [verification](VERIFICATION.md).
In-game checks needed on Windows with the player's own disc data:

- Spam Square while running/crouching after a movable wrench throw, then throw
  again after the catch. Check animation, movement and voice.
- Collect a Gold Bolt, Bolt Grabber and other mapped gadgets on foreign planets;
  check the correct actor/scene, completion, control and music restoration.
- Trade Raritanium for a non-Persuader reward in each supported language; check
  the prompt and subtitles, cost/flag consumption and the actual grant.
- Collect each boots type, immediately travel, die/reload and load a save.
- Damage placed amoeboids of all sizes, then their children/grandchildren using
  direct and indirect weapons. Only the initial actors may advance XP bars.

The original game data and a Windows/GPU game session are unavailable on the
preparation host. These physical presentations and audio are supplied for
playtesting, not claimed as visually verified. Randomizer reachability remains
experimental; this patch does not rearrange existing seeds.
