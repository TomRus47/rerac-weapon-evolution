# Weapon Evolution v0.1.6.40: combat fixes and audio investigation

Compatible base: the modified ReRAC 0.3.4-alpha runtime. Configuration capability: 40.
The existing English weapon profile and launcher 0.5.1 remain compatible; no balancing values or presets are changed.

## Combat findings and changes

- Continuous weapon hit records used Ratchet's runtime body class as their source class. Enemy-specific filters could interpret them as player-body/melee hits or reject them. Independent Pyrocitor/Tesla records now retain ownership and position while identifying the weapon as class 176/177; direct Pyrocitor nozzle hits use the same class as kept flames.
- Gaspar/Quartu pack biters (193) retained a class-specific 60-tick hurt guard and a native flame damage rewrite. Independent attacks bypass that damage gate; nonlethal follow-up hits debit HP and award XP without restarting knockback. Native attacks keep their old rules.
- Gaspar/Batalia hopping gunners (1023), Pokitaru biters (1246) and the existing Quartu guard path must not turn a damage-only result back into a player-body knockback. The two missing guards are now present. Shared source-class normalization also covers other native attacker-class filters.
- Orxon brawlers retained a native cooldown/comet damage rewrite. That rewrite now applies only to native hits, preserving independent configured weapon damage.
- Amoeboids (572/865/866) used a native split-on-hit rule. In real-HP mode each large/medium/small body now owns 300/200/100 HP. A large body at zero activates two medium bodies, each at 200; each medium at zero activates two small bodies, each at 100. Small bodies die at zero. Excess damage is capped for XP and does not skip a size. Children remain damageable during the split throw. Native mode retains its old behavior.
- Devastator range previously measured distance to the moving hero, only checked expiry after an empty collision query, and deleted non-gold missiles without a blast. Range now measures cumulative 3D travel since launch, clips the last collision segment to the remaining distance, and uses the normal damaging explosion at expiry for both variants. Expired gold missiles terminate after that blast; their ordinary collision bounces remain available before expiry. Lifetime expiration also detonates.

## Enemy audit scope

The audit reviewed the shared resolver and all class call sites of `damage::resolve`, plus their class-specific cooldown/attacker filters. Existing quiet-damage paths in critters, flying biters and the Eudora logger, brawler, walker and gunner were checked. Boss phase shields, non-destructible objects, friendly-projectile filters and death states remain native. This is a source audit, not a claim that every encounter has been played in this build.

## Audio findings and open evidence

Rain visuals (weather class 1400) and ambient audio are separate systems. Weather does not play a rain sound directly: native sound-instance emitters and the loaded level sound bank supply ambience. Adding a guessed rain sample to the weather update could duplicate the correct emitter or use the wrong bank index, so this patch does not do that.

Kerwan transport classes 816/1012 already request the native class-816 sound 0 with loop flag 4 while following a path and release it on normal arrival. Invalid/already-completed paths returned before that release; those paths now stop the owned engine loop too. A missing definition, bad bank remap, loop-flag mismatch, range/volume refusal or slot allocation failure can still explain silence. The supplied source snapshot contains no extracted audio banks, so the missing rain/taxi sound is not yet claimed fixed.

## Capture the audio evidence

Set `RC_AUDIO_DIAGNOSTICS` to an absolute writable log filename before launching the existing mod launcher. Its game process inherits the value. The patch's `03_AUDIO_DIAGNOSTICS.bat` does this for you.

The optional report records level bank identity, emitter definitions/ranges, transport sound definitions, play/refusal metadata and emitter state changes sampled once per second. It contains no audio samples or extracted game data and is bounded to 4096 distinct entries per game process. With the variable absent, playback and RNG order are unchanged.

For Batalia: enter a rainy outdoor area, wait ten seconds, enter a sheltered area if available, then exit the game. For a silent taxi: record the planet and route, activate it, ride to the destination, wait five seconds, then exit. Return the diagnostic log and the launcher's `runs/<session>/game.log` for the two cases. This distinguishes an emitter/definition failure from a mixer/playback failure without guessing sound IDs.

## Gameplay checks on Windows

1. Pyrocitor and Tesla: keep contact with Gaspar's hopping gunners and pack biters during recovery; HP/XP should continue at the configured contact DPS/interval without repeated hurt-animation restarts.
2. Amoeboids: use a low-damage stream, a high-damage blast and Doom bots. Confirm 300 -> two separate 200 bodies -> four separate 100 bodies, with XP only for actual HP loss at each stage.
3. Devastator: fire without a target, upwards, while moving, and with gold enabled. The configured travel limit must create an explosion. Put an enemy just inside the blast radius and another outside it; only the reachable enemy should receive blast damage.
4. Boss shields and temporarily disabled turrets: confirm encounter protections and native state transitions still hold.
5. Audio: listen for engine startup/arrival and capture the two missing-sound cases as above.

The Windows game and standalone launcher must be rebuilt before distribution. Source-level regression tests cannot replace these checks.
