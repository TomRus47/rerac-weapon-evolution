# v113 local test — amoeboid Nanotech XP

Runtime: **ReRAC 0.3.8-alpha**. Weapon Evolution **0.1.6.113**,
configuration capability **90**, existing launcher **0.29.3**.
This kit performs no GitHub operations. All v112 integration changes remain.
The upstream audit in `mod/upstream_038.json` records the v112 integration point;
`mod/source-map.json` indexes the current reviewed source bytes.

Weapon Evolution is an independent mod by TomRus47. The author is not associated
with ReRAC development and owns no original-game rights or assets. Thanks to the
ReRAC developers for their continuing work and for making this mod possible.
See [credits](CREDITS.md) and the upstream licenses.

## Cause and behavior

The v111 amoeboid rule intentionally granted a placed small/medium/large actor
100/200/300 weapon XP, independent of the configurable XP-per-damage conversion.
Its shared award function incorrectly passed that fixed weapon budget to
Nanotech as well. This affected all damaging weapons using that branch, not
just the wrench. Swallowing and morphing used an equivalent converted-health
reward and could bypass normal Nanotech conversion too.

The two reward bases are now explicit:

- Weapon XP retains the 100/200/300 placed-actor budget, apportioned by confirmed
  native HP loss. Health scaling does not change this special weapon budget.
- Nanotech uses confirmed effective HP loss multiplied by `xp_per_damage`, as it
  does for other enemies. Enemy health scaling and the active reward-chain
  multiplier apply once. Overkill is capped at remaining HP.
- Split descendants award zero weapon and Nanotech XP. Taunter and Decoy remain
  excluded from Nanotech. Successful swallowing/morphing share the target-life
  special-reward key; repeated callbacks cannot farm XP.

Example with the unchanged default `xp_per_damage = 0.12`, health scale 1 and
reward multiplier x1: a placed large amoeboid supplies 300 weapon XP and **36
Nanotech XP**, rather than 300 Nanotech XP. At health scale 2, spending its 600
effective HP supplies 72 Nanotech XP. Neither generation of split children gives
additional XP. Changing the configured rate still changes Nanotech.

The wrench collision queue already keys hits by attack serial and target.
Repeated animation frames, collision primitives and replayed accepted hit records
cannot grant additional damage or XP for that same instance. Swing, Comet-Strike
and the experimental slam blast have regression coverage. A new attack may hurt
immediately; a slam's configured extra explosion remains its own damage instance,
and only actual accepted HP loss can produce XP.

## Implementation boundaries

`Evolution::award_fixed_xp` receives both the fixed weapon amount and actual native
HP loss with the encounter health scale restored. Internal award helpers accept
separate weapon/Nanotech amounts; ordinary enemies keep their shared conversion.
The resolver, HP setters, attack identities, damage intervals, reaction/stun clocks
and splitting rules are unchanged.

`creature::damage::SpecialReward` captures both bases before a successful morph
can delete the source actor. The same snapshot is used for a confirmed swallow.
An unsuccessful morph still grants nothing. Attraction retains its prior weapon
reward policy and cannot grant Nanotech.

No new settings, GUI changes or default balance edits are required. Native saves,
WEVO v9 companions, seeds and presets retain their formats and values. Previously
earned XP/Nanotech is not rolled back or reconstructed; this affects future XP.
The existing launcher EXE and custom icon remain in place.

## Local installation and checks

Extract the full ZIP into a new folder, close the game, then run
`01_COMPILE_AND_UPDATE.bat`. Select the **existing playable mod folder** with
your launcher EXE, `weapons.ini` and `runtime/rerac.exe`. This kit builds its
bundled sources, tests them and checks identity before transactionally replacing
the runtime and matching shaders. It initializes installed Visual Studio x64
tools. Python, Rust/Cargo, MSVC C++ tools and a Windows SDK are required.

`02_OPEN_UPDATED_LAUNCHER.bat` opens the same updated installation.
`03_CREATE_PLAYABLE_ZIP.bat` packages its current INI, launcher and runtime.
No GUI rebuild or INI migration is needed. Backups and logs follow the
[v112 updater layout](RUNTIME_038.md).

Regression coverage exercises hit delivery, acceptance and HP writes: repeated
wrench instances over 20 frames, fresh attacks, all placed amoeboid sizes, all
15 damaging weapon IDs, health scales 0.5/1/2, overkill, duplicate collision
primitives, successful special-reward snapshots, split descendants, ordinary
numeric/logical enemy HP paths, reward-chain bonuses, the zero-XP switch and the
Nanotech enable switch. Morph-o-Ray is covered through its special reward path.
Existing fire/electricity, robot explosion and failed-morph tests remain.

Automated evidence is in [verification](VERIFICATION.md). Native Windows
compilation and an in-game XP check require the player's machine and game data;
the preparation host cannot claim a Windows/GPU playtest.
