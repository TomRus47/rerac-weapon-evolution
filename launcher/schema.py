"""Field vocabulary from the local Weapon Evolution parser (v0.1.6.51)."""
WEAPONS = {
    'wrench': (8, 'OmniWrench'), 'suck_cannon': (9, 'Suck Cannon'),
    'bomb_glove': (10, 'Bomb Glove'), 'devastator': (11, 'Devastator'),
    'visibomb': (13, 'Visibomb Gun'), 'taunter': (14, 'Taunter'),
    'blaster': (15, 'Blaster'), 'pyrocitor': (16, 'Pyrocitor'),
    'mine_glove': (17, 'Mine Glove'), 'walloper': (18, 'Walloper'),
    'tesla_claw': (19, 'Tesla Claw'), 'glove_of_doom': (20, 'Glove of Doom'),
    'morph_ray': (21, 'Morph-o-Ray'), 'ryno': (23, 'R.Y.N.O.'),
    'drone_device': (24, 'Drone Device'), 'decoy_glove': (25, 'Decoy Glove'),
}
EXTRA = {
    9: 'capacity suck_range_multiplier shot_damage_multiplier',
    11: 'fire_interval_ticks homing_strength_multiplier missile_speed_multiplier missile_range_multiplier explosion_radius_multiplier explosion_visual_size_multiplier explosion_color trail_color',
    13: 'duration_multiplier turn_multiplier invulnerable_while_guiding',
    14: 'lure_range lure_cone_degrees xp_attract_fraction stun_range stun_duration_ticks stun_efficiency_percent damage_range damage_per_second wave_speed_multiplier wave_size_multiplier wave_range_multiplier wave_interval_ticks wave_alpha wave_color',
    15: 'fire_interval_ticks projectile_speed_multiplier projectile_range_multiplier projectile_color trail_color impact_color',
    17: 'throw_range_multiplier trajectory_height_multiplier activation_range_horizontal activation_range_vertical explosion_radius_multiplier',
    18: 'held_lunge_duration_percent knockback_multiplier hit_radius_multiplier electric_fx_size_multiplier gold_electric_fx air_dash_enabled electric_core_color electric_glow_color fist_glow_color impact_color',
    19: 'ammo_interval_ticks damage_interval_ticks target_range_multiplier lock_cone_multiplier beam_core_color beam_glow_color secondary_core_color secondary_glow_color arc_core_color arc_glow_color claw_core_color claw_glow_color spark_color spark_core_color impact_color light_color weapon_glow_color',
    20: 'robot_count robot_max_alive robot_detection_range explosion_radius_multiplier',
    21: 'fire_in_air morph_speed_multiplier beam_range_multiplier explode_small_chickens chicken_damage_multiplier chicken_explosion_radius_multiplier tube_color beam_core_color beam_glow_color secondary_core_color secondary_glow_color pulse_color muzzle_core_color muzzle_glow_color tip_color tip_core_color spark_color spark_core_color light_color',
    23: 'missiles_per_salvo fire_interval_ticks missile_speed_multiplier missile_range_multiplier',
    24: 'drone_count detection_range_multiplier drone_size_multiplier blast_radius_multiplier',
    25: 'decoy_health_multiplier explosion_enabled explosion_radius_multiplier explosion_damage_multiplier throw_range_multiplier xp_attract_fraction lure_range',
}
XP = ('xp_v1_to_v2', 'xp_v2_to_v3', 'xp_v3_to_v4', 'xp_v4_to_v5')
NAMES = {'name', 'name_en', 'name_it', 'name_fr', 'name_de', 'name_es'}
BOOLEANS = set('fire_in_air gold gold_effects gold_texture air_dash_enabled gold_electric_fx invulnerable_while_guiding explode_small_chickens explosion_enabled show_for_weapons_without_ammo level_up_popup'.split())
COLORS = {key for value in EXTRA.values() for key in value.split() if key.endswith('_color')} | set('flame_core_color flame_outer_color burn_flame_color light_color'.split())
BOMB = set('xp_to_next throw_range auto_lock_range trajectory_height explosion_radius explosion_color cooldown_seconds max_ammo damage_multiplier gold gold_effects gold_texture name'.split())
PYRO = set('xp_to_next flame_range flame_width damage_multiplier flame_core_color flame_outer_color light_color max_ammo gold gold_effects gold_texture burn_flame_color'.split())
HUD = set('name_opacity_percent xp_bar_height_px xp_bar_horizontal_inset_px show_for_weapons_without_ammo level_up_popup level_up_popup_duration_seconds level_up_sound'.split())
INTEGER = set('damage_interval_ticks id max_ammo capacity robot_count robot_max_alive drone_count missiles_per_salvo fire_interval_ticks ammo_interval_ticks wave_interval_ticks wave_alpha stun_duration_ticks name_opacity_percent xp_bar_height_px xp_bar_horizontal_inset_px'.split())
BOMB.add('damage_hp')
PYRO.update({'damage_per_second', 'ammo_interval_ticks', 'damage_interval_ticks'})
EDITOR_WEAPONS = {key:value for key,value in WEAPONS.items() if key != "wrench"}
LABELS = {'id': 'Game ID', 'max_ammo': 'Maximum ammo', 'damage_hp': 'Damage per hit (HP)', 'chicken_damage_hp': 'Chicken explosion damage (HP)', 'damage_multiplier': 'Legacy damage multiplier', 'gold': 'Gold behavior and texture (legacy)', 'gold_effects': 'Gold behavior', 'gold_texture': 'Gold texture', 'damage_interval_ticks': 'Damage registration interval', 'fire_interval_ticks': 'Fire interval', 'ammo_interval_ticks': 'Ammo consumption interval', 'projectile_speed_multiplier': 'Projectile speed', 'projectile_range_multiplier': 'Projectile range', 'projectile_color': 'Projectile color', 'trail_color': 'Trail color', 'impact_color': 'Impact color', 'missile_speed_multiplier': 'Missile speed', 'missile_range_multiplier': 'Missile range', 'homing_strength_multiplier': 'Homing strength', 'explosion_radius_multiplier': 'Explosion damage radius', 'explosion_visual_size_multiplier': 'Explosion visual size', 'explosion_color': 'Explosion color', 'throw_range': 'Throw range', 'auto_lock_range': 'Auto-lock range', 'throw_range_multiplier': 'Throw range', 'trajectory_height': 'Trajectory height', 'trajectory_height_multiplier': 'Trajectory height', 'explosion_radius': 'Explosion radius', 'cooldown_seconds': 'Time between throws (seconds)', 'flame_range': 'Flame length', 'flame_width': 'Flame width', 'flame_core_color': 'Flame core color', 'flame_outer_color': 'Outer flame color', 'burn_flame_color': 'Enemy burn flame color', 'light_color': 'Light color', 'capacity': 'Suction capacity', 'suck_range_multiplier': 'Suction range', 'shot_damage_multiplier': 'Legacy shot damage multiplier', 'duration_multiplier': 'Guided missile duration', 'turn_multiplier': 'Turning ability', 'invulnerable_while_guiding': 'Invulnerable while guiding', 'lure_range': 'Attraction range', 'lure_cone_degrees': 'Attraction cone (degrees)', 'xp_attract_fraction': 'Enemy HP fraction used for attraction XP', 'stun_range': 'Stun range', 'stun_duration_ticks': 'Stun duration', 'stun_efficiency_percent': 'Stun chance (%)', 'damage_range': 'Damage range', 'damage_per_second': 'Damage per second (HP/s)', 'wave_speed_multiplier': 'Visual wave speed', 'wave_size_multiplier': 'Visual wave size', 'wave_range_multiplier': 'Visual wave range', 'wave_interval_ticks': 'Visual wave interval', 'wave_alpha': 'Visual wave opacity', 'wave_color': 'Visual wave color', 'activation_range_horizontal': 'Horizontal activation range', 'activation_range_vertical': 'Vertical activation range', 'knockback_multiplier': 'Knockback strength', 'hit_radius_multiplier': 'Hit radius', 'electric_fx_size_multiplier': 'Electric effect size', 'gold_electric_fx': 'Gold electric effects', 'air_dash_enabled': 'Air dash enabled', 'electric_core_color': 'Electric core color', 'electric_glow_color': 'Electric glow color', 'fist_glow_color': 'Fist glow color', 'target_range_multiplier': 'Target range', 'lock_cone_multiplier': 'Lock-on cone width', 'beam_core_color': 'Beam core color', 'beam_glow_color': 'Beam glow color', 'secondary_core_color': 'Secondary beam core color', 'secondary_glow_color': 'Secondary beam glow color', 'arc_core_color': 'Arc core color', 'arc_glow_color': 'Arc glow color', 'claw_core_color': 'Claw core color', 'claw_glow_color': 'Claw glow color', 'spark_color': 'Spark color', 'spark_core_color': 'Spark core color', 'weapon_glow_color': 'Weapon glow color', 'robot_count': 'Robots per throw', 'robot_max_alive': 'Maximum active robots', 'robot_detection_range': 'Robot detection range', 'fire_in_air': 'Fire while airborne', 'morph_speed_multiplier': 'Morph speed', 'beam_range_multiplier': 'Beam range', 'explode_small_chickens': 'Explode small chickens', 'chicken_damage_multiplier': 'Legacy chicken damage multiplier', 'chicken_explosion_radius_multiplier': 'Chicken explosion radius', 'tube_color': 'Tube color', 'pulse_color': 'Pulse color', 'muzzle_core_color': 'Muzzle core color', 'muzzle_glow_color': 'Muzzle glow color', 'tip_color': 'Beam tip color', 'tip_core_color': 'Beam tip core color', 'missiles_per_salvo': 'Missiles per salvo', 'drone_count': 'Drone count', 'detection_range_multiplier': 'Drone detection range', 'drone_size_multiplier': 'Drone size', 'blast_radius_multiplier': 'Drone blast radius', 'decoy_health_multiplier': 'Decoy health', 'explosion_enabled': 'Explosion enabled', 'explosion_damage_multiplier': 'Legacy explosion damage multiplier', 'xp_per_damage': 'XP per HP of damage', 'name_opacity_percent': 'Weapon name opacity (%)', 'xp_bar_height_px': 'XP bar height (pixels)', 'xp_bar_horizontal_inset_px': 'Weapon name horizontal inset (pixels)', 'show_for_weapons_without_ammo': 'Show HUD for weapons without ammo', 'level_up_popup': 'Level-up notification', 'level_up_popup_duration_seconds': 'Level-up notification duration (seconds)', 'level_up_sound': 'Level-up sound', 'xp_to_next': 'XP to next grade (legacy)'}
HELP = {'damage_hp': 'HP removed by one projectile or explosion. The same instance hits each enemy once; another instance can hit immediately.', 'chicken_damage_hp': 'HP removed by a Morph-o-Ray chicken explosion.', 'damage_per_second': 'HP per second of actual stream contact, grouped using damage_interval_ticks. Visual particles and beam segments do not multiply damage.', 'max_ammo': 'Ammo capacity at this grade. Loading a configuration does not grant free ammo.', 'gold_effects': 'Enables the original Gold weapon mechanics and bonuses independently of its texture.', 'gold_texture': 'Uses the Gold appearance for the 3D weapon model. Does not enable Gold gameplay bonuses.', 'gold': 'Legacy setting controlling Gold behavior and texture together. Prefer the two separate settings.', 'auto_lock_range': 'default follows the useful throw range. A number can limit auto-lock distance without increasing throw range.', 'lure_range': 'Attraction range. For the Decoy Glove, default preserves each enemy’s native detection distance.', 'xp_attract_fraction': 'Fraction of the enemy’s remaining HP used to award XP on attraction. 0.333333 is approximately one third.', 'wave_speed_multiplier': 'Taunter visual wave speed. Does not increase damage, attraction or stun.', 'wave_size_multiplier': 'Visual wave size. Does not change the area of effect.', 'wave_range_multiplier': 'Visual wave reach. Does not change attraction or damage range.', 'wave_alpha': 'Native wave opacity: 0 = invisible, 128 = full opacity. Original value: 20.', 'explosion_visual_size_multiplier': 'Changes only the visual effect. Damage radius is a separate setting.', 'ammo_interval_ticks': 'Consumes one ammo unit every N firing ticks. Independent of damage; higher values consume ammo more slowly.', 'damage_interval_ticks': 'Registers damage and XP every N actual contact ticks with each enemy (1–600). Only contact contributes, so longer intervals do not increase HP/s. Partial damage waits for the next pulse and resets when the target is removed or a game is loaded.', 'fire_in_air': 'Allows the Morph-o-Ray to fire while airborne at this grade. Running fire is always allowed.', 'level_up_sound': 'help_notice = notification sound; off = silent.', 'name_opacity_percent': 'Opacity of the HUD weapon name, grade and their black background: 100 = normal, 50 = semi-transparent, 0 = hidden. Does not change the 3D weapon model or the XP bar.', 'xp_bar_horizontal_inset_px': 'Horizontal inset for the weapon name. The XP bar remains aligned with the outside edges of the ammo panel.', 'id': 'Game identifier, locked in the editor. In-game weapon names follow the game language.', 'xp_per_damage': 'XP awarded per HP actually removed from eligible enemies. For example, 0.12 gives 12 XP for 100 HP; overkill does not grant extra XP.'}

for i,key in enumerate(XP,1): LABELS[key] = f'XP from V{i} to V{i+1}'

def title(key):
    return LABELS.get(key,key.replace('_',' ').capitalize())

def help_text(section,key):
    text=HELP.get(key,'')
    if key=='damage_per_second' and section.startswith('weapon.taunter.'):
        text='Taunter offensive aura damage in HP/s within damage_range. Separate from attraction and stun.'
    if key in COLORS:
        text='RGB color: RRGGBB or #RRGGBB. default keeps the original color. Use the swatch to choose a color.'
    elif key in XP:
        text='XP required for this grade transition, not a cumulative total. Does not change XP already stored in saves.'
    elif key.endswith('_ticks'):
        text=(text+' ' if text else '')+'Game ticks: 60 ticks = 1 second, independently of rendering FPS.'
    elif key.endswith('_multiplier') and not text:
        text='Multiplier of the reference behavior: 1 = reference, 0.5 = half, 2 = double.'
    elif key in ('throw_range','trajectory_height','explosion_radius','flame_range','flame_width'):
        text='Multiplier of the reference weapon setting; 1 preserves the reference value.'
    return f'[{section}] {key}\n{text or "Value read directly from this mod’s INI configuration."}'


# Configuration policy: standard fields first; experimental switches and their
# parameters always last, for every grade and every future weapon experiment.
STANDARD_DEFAULTS = {
    'walloper': {'held_lunge_duration_percent': '100'},
    'tesla_claw': {'beam_size_percent': '100', 'beam_range_multiplier': '1'},
}
EXPERIMENTAL_DEFAULTS = {
    'bomb_glove': {'cluster_bombs_enabled': 'false', 'cluster_bomb_count': '3',
        'cluster_bomb_size_percent': '40', 'cluster_explosion_size_percent': '15',
        'cluster_damage_percent': '10', 'cluster_launch_speed': '3', 'cluster_jump_speed': '4.2'},
    'pyrocitor': {'triple_flame_enabled': 'false', 'side_flame_left_angle_degrees': '22',
        'side_flame_right_angle_degrees': '22'},
    'suck_cannon': {'crate_suction_enabled': 'false', 'suck_wooden_crates': 'true',
        'suck_explosive_crates': 'true', 'suck_steel_crates': 'true',
        'suck_ammo_crates': 'true', 'suck_nanotech_crates': 'true'},
    'tesla_claw': {'dual_beam_enabled': 'false'},
}

# Gold appearance is independent of native Gold mechanics, including weapons
# which had no purchasable Gold variant in the original game.
for weapon in ('visibomb', 'taunter', 'walloper', 'ryno'):
    STANDARD_DEFAULTS.setdefault(weapon, {}).setdefault('gold_texture', 'false')
GLOBAL_STANDARD_DEFAULTS = {'wrench_gold_texture': 'false'}
BOOLEANS.update(GLOBAL_STANDARD_DEFAULTS)
LABELS['wrench_gold_texture'] = 'OmniWrench Gold texture'
HELP['wrench_gold_texture'] = 'Gold appearance for the OmniWrench metal surface, without changing its damage or introducing weapon grades.'
GLOBAL_EXPERIMENTAL_DEFAULTS = {'experimental_ammo_unlock_mode': 'false'}
BOOLEANS.update(GLOBAL_EXPERIMENTAL_DEFAULTS)
LABELS['experimental_ammo_unlock_mode'] = 'Experimental ammo-unlock mode: on/off'
HELP['experimental_ammo_unlock_mode'] = ('Unlock ammo weapons while ammo remains; each unlock inserts the quick-select icon. '
    'No weapon/ammo shops. Ammo crates drop 1-4 independent packs, including unowned weapons; Gold Bolts improve pack count. '
    'Taunter unlocks at 25,000 collected normal bolts, Suck Cannon at 50,000, Morph-o-Ray at 75,000. '
    'Walloper is available immediately. State is saved per slot. Disabling restores native ownership.')

# v45 additions remain in the same grade, with experiments at the footer.
STANDARD_DEFAULTS.setdefault('ryno', {}).update({'homing_strength_multiplier': '1', 'missile_max_range': '0', 'explosion_damage_radius': '0', 'explosion_visual_size_multiplier': '1'})
STANDARD_DEFAULTS.setdefault('mine_glove', {}).update({'max_active_mines': '8', 'mine_lifetime_seconds': '30', 'flame_core_color': 'default', 'flame_outer_color': 'default', 'burn_flame_color': 'default', 'explosion_color': 'default', 'light_color': 'default'})
STANDARD_DEFAULTS.setdefault('taunter', {}).update({'stun_launch_distance': '0'})
EXPERIMENTAL_DEFAULTS.setdefault('ryno', {}).update({'continuous_missiles_enabled': 'false', 'shotgun_enabled': 'false', 'shotgun_damage_hp': '100', 'shotgun_speed': '40', 'shotgun_range': '30', 'shotgun_size_percent': '100', 'shotgun_horizontal_spread_degrees': '12', 'shotgun_vertical_spread_degrees': '3', 'shotgun_projectile_color': 'default', 'shotgun_trail_color': 'default', 'shotgun_impact_color': 'default'})
EXPERIMENTAL_DEFAULTS.setdefault('mine_glove', {}).update({'flame_mine_enabled': 'false', 'mine_flame_range': '8', 'mine_flame_rotation_degrees_per_second': '90', 'mine_flame_damage_per_second': '100'})
EXPERIMENTAL_DEFAULTS.setdefault('drone_device', {}).update({'sand_mouse_enabled': 'false', 'sand_mouse_mode': '1', 'sand_mouse_lifetime_seconds': '90', 'sand_mouse_max_shots': '100', 'sand_mouse_fire_interval_ticks': '8', 'sand_mouse_shot_range': '24', 'sand_mouse_shot_damage_hp': '50', 'sand_mouse_projectile_color': 'default'})
EXPERIMENTAL_DEFAULTS.setdefault('tesla_claw', {}).update({'secondary_beam_damage_percent': '25'})

for weapon, fields in EXPERIMENTAL_DEFAULTS.items():
    if weapon == 'bomb_glove': BOMB.update(fields)
    elif weapon == 'pyrocitor': PYRO.update(fields)
    else: EXTRA[WEAPONS[weapon][0]] += ' ' + ' '.join(fields)
for weapon, fields in STANDARD_DEFAULTS.items():
    EXTRA[WEAPONS[weapon][0]] += ' ' + ' '.join(fields)
BOOLEANS.update({'cluster_bombs_enabled', 'triple_flame_enabled', 'crate_suction_enabled',
    'suck_wooden_crates', 'suck_explosive_crates', 'suck_steel_crates',
    'suck_ammo_crates', 'suck_nanotech_crates', 'dual_beam_enabled'})
INTEGER.add('cluster_bomb_count')
LABELS.update({
    'cluster_bombs_enabled': 'Experimental cluster bombs: on/off',
    'cluster_bomb_count': 'Mini bombs per explosion',
    'cluster_bomb_size_percent': 'Mini bomb size (%)',
    'cluster_explosion_size_percent': 'Mini explosion size (%)',
    'cluster_damage_percent': 'Mini bomb damage (%)',
    'cluster_launch_speed': 'Mini bomb forward speed (units/s)',
    'cluster_jump_speed': 'Mini bomb initial jump speed (units/s)',
    'triple_flame_enabled': 'Experimental triple flame: on/off',
    'side_flame_left_angle_degrees': 'Left flame angle (degrees)',
    'side_flame_right_angle_degrees': 'Right flame angle (degrees)',
    'held_lunge_duration_percent': 'Held punch duration (%)',
    'crate_suction_enabled': 'Experimental crate suction: on/off',
    'suck_wooden_crates': 'Allow wooden crates',
    'suck_explosive_crates': 'Allow explosive crates',
    'suck_steel_crates': 'Allow steel crates',
    'suck_ammo_crates': 'Allow ammo crates',
    'suck_nanotech_crates': 'Allow nanotech crates',
    'beam_size_percent': 'Beam size (%)', 'beam_range_multiplier': 'Beam reach multiplier',
    'dual_beam_enabled': 'Experimental independent dual beams: on/off',
})
HELP.update({
    'cluster_bombs_enabled': 'Release visible mini bombs before a surface-impact blast. Mini bombs never create further mini bombs.',
    'cluster_bomb_count': 'Mini bombs per parent explosion, 1-12. Default: 3.',
    'cluster_bomb_size_percent': 'Model size relative to the parent bomb. Default: 40%.',
    'cluster_explosion_size_percent': 'Visual explosion and damage radius as a percentage of the parent blast. Default: 15%.',
    'cluster_damage_percent': 'Damage relative to this grade, snapshotted when released. Default: 10%.',
    'cluster_launch_speed': 'Horizontal speed along each mini bomb direction, in game units per second. Default: 3, half the old default.',
    'cluster_jump_speed': 'Initial upward velocity in game units per second. Gravity stays native; contact detonation arms at the apex. Default: 4.2.',
    'triple_flame_enabled': 'Two additional jets share the nozzle, ammo clock and HP/s. Overlap does not multiply damage on one enemy.',
    'side_flame_left_angle_degrees': 'Angle left of the central jet, 0-80 degrees. Default: 22.',
    'side_flame_right_angle_degrees': 'Angle right of the central jet, 0-80 degrees. Default: 22.',
    'held_lunge_duration_percent': '100 is native. Higher values extend propulsion and the hit window while fire stays held, once per enemy per punch.',
    'crate_suction_enabled': 'Use allowed crates as native-slot ammunition. Preserve their model and contents until impact. Steel deals double projectile damage; TNT blast damage earns Suck Cannon XP.',
    'suck_wooden_crates': 'Wooden crates use normal projectile damage and release their stored bolts when destroyed.',
    'suck_explosive_crates': 'Explosive crates detonate on impact. Their blast uses this grade damage and awards actual-loss XP.',
    'suck_steel_crates': 'Steel crates deal twice this grade projectile damage and break on any surface, releasing stored bolts.',
    'suck_ammo_crates': 'Ammo crates release their original ammo reward when destroyed after firing.',
    'suck_nanotech_crates': 'Nanotech stays inside the carried crate and is released only after the fired crate is destroyed.',
    'beam_size_percent': 'Overall player beam width. 100 is native. Does not change HP/s or reach.',
    'beam_range_multiplier': 'Scales visible and damaging reach. For Tesla, also multiplies its existing target range setting.',
    'dual_beam_enabled': 'Two simultaneous rays reserve distinct enemies. Each deals normal damage. With only one enemy, only one ray can damage it. Ammo is charged once.',
})
EXPERIMENTAL_RANGES = {
    'cluster_bomb_count': (1, 12), 'cluster_bomb_size_percent': (1, 100),
    'cluster_explosion_size_percent': (1, 100), 'cluster_damage_percent': (.1, 100),
    'cluster_launch_speed': (0, 20), 'cluster_jump_speed': (.1, 10),
    'side_flame_left_angle_degrees': (0, 80), 'side_flame_right_angle_degrees': (0, 80),
    'held_lunge_duration_percent': (100, 1000), 'beam_size_percent': (1, 500),
    'beam_range_multiplier': (.1, 8),
}

def field_groups(weapon, keys):
    experimental = EXPERIMENTAL_DEFAULTS.get(weapon, {})
    return ([k for k in keys if k not in experimental], [k for k in experimental if k in keys])

# Shared validation also covers the new standard controls.
EXPERIMENTAL_RANGES.update({'homing_strength_multiplier': (0, 8), 'missile_max_range': (0, 500), 'explosion_damage_radius': (0, 100), 'explosion_visual_size_multiplier': (0.1, 8), 'max_active_mines': (1, 32), 'mine_lifetime_seconds': (0.5, 600), 'stun_launch_distance': (0, 50), 'shotgun_damage_hp': (0, 9999), 'shotgun_speed': (1, 200), 'shotgun_range': (0.5, 500), 'shotgun_size_percent': (1, 500), 'shotgun_horizontal_spread_degrees': (0, 60), 'shotgun_vertical_spread_degrees': (0, 60), 'mine_flame_range': (0.5, 100), 'mine_flame_rotation_degrees_per_second': (0, 720), 'mine_flame_damage_per_second': (0, 9999), 'sand_mouse_mode': (1, 3), 'sand_mouse_lifetime_seconds': (0.5, 600), 'sand_mouse_max_shots': (1, 9999), 'sand_mouse_fire_interval_ticks': (1, 9999), 'sand_mouse_shot_range': (0.5, 500), 'sand_mouse_shot_damage_hp': (0, 9999), 'secondary_beam_damage_percent': (0, 100)})
BOOLEANS.update({'continuous_missiles_enabled','shotgun_enabled','flame_mine_enabled','sand_mouse_enabled'})
INTEGER.update({'max_active_mines','sand_mouse_mode','sand_mouse_max_shots','sand_mouse_fire_interval_ticks'})
COLORS.update({k for group in (STANDARD_DEFAULTS,EXPERIMENTAL_DEFAULTS) for fields in group.values() for k in fields if k.endswith('_color')})
HELP.update({'continuous_missiles_enabled': 'Fire one missile per interval while fire is held. One ammo per missile. Replaces the native salvo; cannot be combined with shotgun mode.', 'shotgun_enabled': 'Fire 35 independent Blaster-shaped pellets in a seven-column, five-row muzzle grid. One ammo per volley. Replaces all native missiles; cannot be combined with continuous missiles.', 'shotgun_damage_hp': 'Damage of each of the 35 pellets, independently snapshotted at firing. Each pellet hits an enemy once; damage is not divided among pellets.', 'shotgun_horizontal_spread_degrees': 'Maximum sideways angle at the outer columns (degrees). Intermediate columns scale proportionally; the centre stays straight. 0 keeps all columns parallel horizontally.', 'shotgun_vertical_spread_degrees': 'Maximum upward/downward angle at the outer rows (degrees). The intermediate rows use half this angle. 0 keeps rows parallel vertically. Both axes at 0 retain 35 separate origins in a rectangular grid.', 'missile_max_range': 'Travel distance from the muzzle, in game units. 0 preserves the native envelope. Range expiry detonates the missile.', 'explosion_damage_radius': 'R.Y.N.O. missile area damage radius in game units. 0 preserves native direct-impact-only damage. Visual size is independent.', 'max_active_mines': 'Maximum deployed mines. Launching another retires the oldest without a damaging blast.', 'mine_lifetime_seconds': 'Time after release until automatic retirement, including flight time. Applies to normal and experimental mines.', 'flame_mine_enabled': 'After landing, replace native activation, pursuit and blast damage with four rotating flame arms. Each mine has its own spatial flame loop and earns Mine Glove XP.', 'mine_flame_damage_per_second': 'Actual HP per second of flame contact. Multiple arms from the same mine never multiply damage on one enemy; separate mines are independent.', 'mine_flame_rotation_degrees_per_second': 'Rotation speed of the four flame arms. 0 keeps a fixed cross.', 'stun_launch_distance': 'Additional collision-checked push on a successful Taunter stun. 0 keeps the original recoil.', 'sand_mouse_enabled': 'Summon the Sonic Summoner flying companion using Drone Device ammo. Its shots award Drone Device XP. Only one mouse can be active, including native dens.', 'sand_mouse_mode': '1 = companion replaces drones; 2 = companion and drones together; 3 = companion while the Sonic Summoner is worn, normal drones otherwise. An active companion blocks another launch without using ammo.', 'sand_mouse_lifetime_seconds': 'Companion lifetime, independent of owning, equipping or removing the Sonic Summoner. It grows from 1% to native size in 0.5 seconds.', 'sand_mouse_max_shots': 'Retire the companion after this many successfully created shots, even before its lifetime expires.', 'sand_mouse_shot_range': 'Maximum shot travel distance and maximum acquisition distance, in game units.', 'secondary_beam_damage_percent': 'Second native electrical ray damage when it hits the same enemy as the first: 25 = 25% of base HP/s. Against a different enemy it deals full base HP/s. Ammo is charged once.', 'dual_beam_enabled': 'Two independent native electrical rays, including free firing with no targets. The second prefers another enemy, but may hit the first at secondary_beam_damage_percent.'})
LABELS.update({'shotgun_horizontal_spread_degrees':'Horizontal shotgun spread (degrees)','shotgun_vertical_spread_degrees':'Vertical shotgun spread (degrees)','sand_mouse_enabled':'Experimental Sand Mouse: on/off','sand_mouse_mode':'Companion mode (1 / 2 / 3)','secondary_beam_damage_percent':'Second ray overlap damage (%)','continuous_missiles_enabled':'Experimental continuous missiles: on/off','shotgun_enabled':'Experimental shotgun: on/off','flame_mine_enabled':'Experimental flame mine: on/off'})

for group in (STANDARD_DEFAULTS,EXPERIMENTAL_DEFAULTS):
    for fields in group.values():
        for key in fields:
            LABELS.setdefault(key,key.replace('_',' ').capitalize())
            HELP.setdefault(key, {
                'shotgun_speed':'Pellet speed in game units per second, independent of the Blaster.',
                'shotgun_range':'Maximum pellet travel distance in game units, independent of the Blaster.',
                'shotgun_size_percent':'Pellet model, glow and trail size. 100 = native Blaster size.',
                'mine_flame_range':'Length of each flame arm in game units, clipped by world geometry.',
                'sand_mouse_fire_interval_ticks':'Minimum interval between successfully created companion shots.',
                'sand_mouse_shot_damage_hp':'HP removed by each companion shot. Actual enemy HP loss awards Drone Device XP.',
                'homing_strength_multiplier':'Missile steering strength. 0 disables steering; 1 preserves native steering.'
            }.get(key,'Per-grade setting; stored separately for this weapon.'))

HELP['drone_count'] = 'Maximum simultaneous shield drones at this grade (0–10). Above six, a shared orbit expands to keep drones spaced apart.'
