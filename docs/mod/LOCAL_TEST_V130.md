# Local test v130 — Taunter modes, PDA refills and randomizer sites

ReRAC **0.3.8-alpha**, Weapon Evolution **0.1.6.130**, configuration capability
**95**, launcher **0.29.8**. This is a standalone local test. Nothing is uploaded
to GitHub and no release is created. All v129 gameplay, loading, audio, water,
HUD and dual Quick Select fixes are retained.

## Install and test

Extract this whole kit into a new folder. Close the game and the mod launcher.
Run **01_COMPILE_AND_UPDATE.bat** and select the existing playable mod folder
containing weapons.ini, ReRAC_Weapon_Evolution.exe and runtime/rerac.exe.
Do not select this source kit or the old ReRAC source checkout.

The kit includes complete verified source. It builds and checks the runtime
**and** the updated launcher, retaining the custom icon, personal balance,
presets, native saves and randomizer placements. Successful staging replaces
the installed files transactionally; a failed build keeps the previous playable
executables. Visual Studio x64 tools and the existing isolated Rust recovery
are reused. Read logs/update-error.log if the operation fails.

Use **02_OPEN_UPDATED_LAUNCHER.bat** to open that same installation.
**03_CREATE_PLAYABLE_ZIP.bat** creates a player ZIP from the updated installation
and its current configuration, excluding game data, personal saves and presets.

### Taunter

Each grade has an **Offensive taunt** Off/On control. Damage per second, damage
radius, stun radius/duration/chance and launch distance are grouped with it in
the experimental frame. Attraction range, cone, attraction XP and wave visuals
remain standard. Existing offensive tuning values are retained.

With global **R1 Experimental On**, saved mode **0 only attracts**, while mode
**1 adds the configured damage, stun and launch**. Pure attraction never delivers
a recoil hit. Each grade's **R1_Experimental max** controls availability: 0 hides
and locks the switch; 1 allows 0–1. Existing zero placeholders from older profiles
are upgraded once to 1, since those profiles could not select a Taunter mode.
Subsequent explicit choices remain unchanged. With global R1 Off, the launcher
switch applies directly; its legacy-compatible initial setting is On.

Test Orxon's force-field skill point in mode 0, then check mode 1 independently.
Attraction XP remains once per eligible enemy life, with its existing fraction
and boss rules. Neither Taunter mode grants Nanotech XP.

### Pause-menu names and XP

Gold flags no longer assume every weapon has a native gold-name table row.
The name label uses the disc's localized ordinary name when its gold row is
missing or invalid, then appends the actual grade. Other labels and descriptions
are left alone. Restoring name-panel recognition also restores the XP bar below
it. Test Drone Device, Taunter, Walloper, Visibomb and R.Y.N.O. at V6–V8.
The shared path retains localized bytes for every game language.

### PDA: buy all ammunition

Square now buys all missing owned-weapon ammunition through the PDA too.
The total uses **PDA unit prices**, active per-grade/mode capacities and existing
ownership restrictions. The quote is recalculated on the input edge. Insufficient
funds cannot partially refill or spend bolts. The localized hint and total also
appear above the PDA carousel.

### Randomizer All: grey bolt packs

Unused native Gold Bolt sites and enabled extra sites contain a grey Gold Bolt
model. Collecting it shows a localized **Bolt pack obtained!** message and grants
standard bolts once. **Bolt pack value** in the Randomizer card defaults to
**5000** and supports live edits. Packs do not increase Gold Bolt totals, weapon
XP or Nanotech XP, and do not apply the reward-chain multiplier. There is no
collection cutaway. Existing seed reward placements are not replaced.

Packs use the same saved location receipt as other randomizer rewards. Native
Gold Bolt models/textures and normal/Weapons-only games are unaffected. The grey
treatment is local to the displayed model, including its metallic surfaces.

### Six independent extra-site switches

The Randomizer card has one Off/On switch for each extra site on **Gemlik Base,
Kerwan, Orxon, Pokitaru, Hoven and Novalis**. Exact positions remain undisclosed
in launcher help. All six start enabled. These switches affect **new All seeds
only**, not Weapons-only mode, existing saves, live loaded worlds or NG+ continuation.

The exact six-bit choice is saved with new seeds. Legacy saves retain all six
candidate sites. Loading always uses the saved choice even if the launcher has
since been changed. Disabled extra sites contain neither a reward nor a pack.
Generation still validates progression, coordinate pacing and the pre-Gemlik
weapon requirement. With all extras disabled, the extra starter-weapon reward
replaces one non-progression Gold Bolt filler so the finite native pool fits;
all equipment and coordinate rewards remain present. The authored route model
is unchanged. Code-level checks do not guarantee every physical mission path.

### NG+ enemy reports

The earlier Veldin fix is included: its shared electrical-beam manager runs once
per native tick instead of consuming freshly published beams on an accelerated
second update. Enemy AI still uses its configured speed.

Additional tests exercise actual Blarg trooper/fire-wave bot AI timers at
1, 1.25, 2 and 4 times speed with the NG+ multiplier applied, and preserve the
crawler's native Ratchet/Clank-side sleep gate. Those checks do not reproduce an
immobile enemy. No blanket speed, damage, phase or mission-gate workaround is
applied. For a remaining stationary enemy, report its exact room/type and settings
or a short video; an entrance-animation/mission-path failure needs that context.

## Save compatibility and implementation

The native save format is unchanged. WEVO **v11** appends two bytes after the
v10 dual-ring payload: Taunter mode and the randomizer extra-site mask. It retains
all preceding field offsets, content-addressed native-save binding and SHA1
checksum. v1–v10 companions still load; v10 retains both rings, its active ring,
all preceding R1 choices, seed, placements and claimed rewards. New companions
require this test runtime; do not write the same slots with an older runtime.

The randomizer's new generated layout version is 9. Empty-site packs are derived
from the saved location catalogue/mask and keep their receipt bit; they are not
inserted into the progression reward pool. Already-earned XP is not rewritten.

See VERIFICATION.md for automated checks and their practical limits. Windows
linking, frozen GUI self-tests and GPU appearance are verified by the local build
and in-game test; this preparation host does not run the Windows game.

Weapon Evolution is an independent mod by **TomRus47**, who is not associated
with development of ReRAC or the original Ratchet & Clank games. The author
claims no original-game rights, assets or trademarks. Thanks to the ReRAC
team: their port and continuing work make these mods possible. Original-game
rights and upstream licenses/credits remain with their respective owners.
No original game data is included.
