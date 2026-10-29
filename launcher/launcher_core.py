"""Lossless presets and explicit one-process ReRAC configuration. Python 3.10+."""
from __future__ import annotations
import hashlib
from nanotech_cap_profile import migrate_health_cap
import json
import math
import os
import re
import shutil
import struct
import subprocess
import tempfile
import uuid
import zipfile
import sys
from frozen_support import application_folder, native_process_context, native_environment
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from schema import WEAPONS, XP, NAMES, BOOLEANS, COLORS, BOMB, PYRO, HUD, INTEGER, EXTRA

VERSION = '0.29.5'
KIT = application_folder(__file__)
PREVIOUS = (
    'ReRAC_Weapon_Evolution_v0.1.6.36_Fix_Munizioni_Negozio_Editabile',
    'ReRAC_Weapon_Evolution_v0.1.6.35_Camera_Fluida_Editabile',
    'ReRAC_Weapon_Evolution_v0.1.6.34_Camera_R2_Respawn_Editabile',
    'ReRAC_Weapon_Evolution_v0.1.6.33_Mirini_Camera_Fade_Editabile',
)
SECTION = re.compile(r'^\s*\[([^\]]+)\]\s*$')
ASSIGN = re.compile(r'^(\s*[^=]+?\s*=\s*)(.*?)(\s*)$')
COLOR = re.compile(r'#?[0-9a-fA-F]{6}\Z')
NUMBER = re.compile(r'[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?\Z')
LEVEL = re.compile(r'weapon\.([a-z_]+)\.level([1-8])\Z')

def atomic_write(path: Path, data: bytes):
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix='.rerac-write-', dir=path.parent)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(data); stream.flush(); os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        Path(temporary).unlink(missing_ok=True)

def stamp():
    return datetime.now().strftime('%Y%m%d-%H%M%S-%f') + '-' + uuid.uuid4().hex[:6]

class IniDocument:
    """Preserve BOM, comments, layout, line endings and unedited values exactly."""
    def __init__(self, raw: bytes):
        self.raw = raw
        self.bom = raw.startswith(b'\xef\xbb\xbf')
        self.lines = raw.decode('utf-8-sig').splitlines(keepends=True)
        self.entries: dict[tuple[str, str], str] = {}
        self.comments: dict[tuple[str, str], str] = {}
        self.sections: list[str] = []
        self.addresses = {}
        section = ''; comments = []
        for i, line in enumerate(self.lines):
            stripped = line.split(';', 1)[0].strip()
            if not stripped or stripped.startswith('#'):
                if line.lstrip().startswith((';', '#')):
                    comments.append(line.lstrip()[1:].strip())
                continue
            header = SECTION.fullmatch(stripped)
            if header:
                section = header[1].strip().lower()
                if section in self.sections: raise ValueError(f'Duplicate section: [{section}]')
                self.sections.append(section); comments = []; continue
            if '=' not in stripped or not section:
                raise ValueError(f'Line {i+1}: expected a section or key = value.')
            key, value = stripped.split('=', 1); key = key.strip().lower(); value = value.strip()
            pair = section, key
            if pair in self.entries: raise ValueError(f'Duplicate setting: [{section}] {key}')
            self.entries[pair] = value
            self.comments[pair] = ' '.join(comments[-3:]); comments = []
            self.addresses[pair] = i

    def keys(self, section):
        return [key for sec, key in self.entries if sec == section]

    def render(self, edits: dict[tuple[str, str], str] | None = None):
        lines = self.lines.copy()
        for pair, value in (edits or {}).items():
            if pair not in self.entries: raise ValueError(f'Missing setting: {pair}')
            if value == self.entries[pair]: continue
            if any(char in value for char in '\r\n;'):
                raise ValueError('Values cannot contain extra lines or comments.')
            i = self.addresses[pair]; line = lines[i]
            end = '\r\n' if line.endswith('\r\n') else '\n' if line.endswith('\n') else '\r' if line.endswith('\r') else ''
            body = line[:-len(end)] if end else line
            code, sep, comment = body.partition(';')
            match = ASSIGN.fullmatch(code)
            if not match: raise ValueError(f'Cannot edit line: {pair}')
            lines[i] = match[1] + value + match[3] + sep + comment + end
        return (b'\xef\xbb\xbf' if self.bom else b'') + ''.join(lines).encode('utf-8')

def allowed_keys(section):
    if section == 'general':
        from schema import GLOBAL_EXPERIMENTAL_DEFAULTS, GLOBAL_STANDARD_DEFAULTS
        return {'xp_per_damage', 'damage_mode', *GLOBAL_EXPERIMENTAL_DEFAULTS, *GLOBAL_STANDARD_DEFAULTS}
    if section == 'hud': return HUD
    if section.startswith('weapon.') and section[7:] in WEAPONS:
        return {'id', *XP, *(NAMES-{'name'})}
    match = LEVEL.fullmatch(section)
    if not match or match[1] not in WEAPONS:
        raise ValueError(f'Unsupported configuration section: [{section}]')
    name = match[1]
    if name == 'bomb_glove': return BOMB
    if name == 'pyrocitor': return PYRO
    extra = {'damage_per_second'} if name == 'tesla_claw' else {'chicken_damage_hp'} if name == 'morph_ray' else {'damage_hp'}
    return set(EXTRA.get(WEAPONS[name][0], '').split()) | extra | {'max_ammo', 'damage_multiplier', 'gold', 'gold_effects', 'gold_texture'}

def numeric_range(section, key):
    if key=="r1_experimental_max":
        from schema import r1_mode_options
        return 0,int(r1_mode_options(section)[-1])
    from schema import WORLD_RANGES
    if section=="general" and key in WORLD_RANGES:return WORLD_RANGES[key]
    if section.startswith("weapon.wrench.level") and key in {"hit_radius_multiplier","knockback_multiplier"}: return (0.1,10) if key=="hit_radius_multiplier" else (0,10)
    from schema import EXPERIMENTAL_RANGES
    if section.startswith("weapon.wrench.level") and key == "throw_range": return 0.5,100
    if key == 'experimental_max_ammo': return (0,10) if section.startswith('weapon.suck_cannon.level') else (1,999) if section.startswith(('weapon.bomb_glove.level','weapon.pyrocitor.level')) else (0,9999)
    if key in EXPERIMENTAL_RANGES and (key != "beam_range_multiplier" or section.startswith("weapon.tesla_claw.level")) and (key != "homing_strength_multiplier" or section.startswith("weapon.ryno.level")): return EXPERIMENTAL_RANGES[key]
    if key == 'damage_interval_ticks': return 1, 600
    if key == 'ammo_interval_ticks': return 1, 9999
    if key in {'damage_hp','chicken_damage_hp'} or (key == 'damage_per_second' and section.startswith(('weapon.pyrocitor.level','weapon.tesla_claw.level'))): return 0, 1_000_000
    if key in XP or key == 'xp_to_next': return .1, 1_000_000
    if section == 'general':
        from schema import AMMO_UNLOCK_DEFAULTS
        return (0, 1_000_000_000) if key in AMMO_UNLOCK_DEFAULTS else (0, 100)
    if section == 'hud':
        return {'nanotech_xp_bar_opacity_percent': (0, 100), 'name_opacity_percent': (0, 100), 'xp_bar_height_px': (2, 16),
                'xp_bar_horizontal_inset_px': (0, 24), 'level_up_popup_duration_seconds': (.5, 15),
                'xp_bar_opacity_percent': (0, 100)}.get(key, (0, 9999))
    if section.startswith('weapon.bomb_glove.level'):
        return {'throw_range': (.1, 5), 'trajectory_height': (.4, 3), 'explosion_radius': (.1, 8),
                'cooldown_seconds': (.1, 10), 'max_ammo': (1, 999), 'damage_multiplier': (.1, 20)}.get(key, (0, 9999))
    if section.startswith('weapon.pyrocitor.level'):
        return {'flame_range': (.1, 3), 'flame_width': (.1, 3), 'max_ammo': (1, 999),
                'damage_multiplier': (.1, 20)}.get(key, (0, 9999))
    if key in {'wave_speed_multiplier', 'wave_size_multiplier', 'wave_range_multiplier', 'explosion_visual_size_multiplier'} or (section.startswith('weapon.devastator.level') and key == 'explosion_radius_multiplier'):
        return .1, 8
    return {'wave_interval_ticks': (1, 600), 'wave_alpha': (0, 128), 'stun_efficiency_percent': (0, 100),
            'lure_cone_degrees': (1, 180), 'stun_duration_ticks': (0, 600),
            'capacity': (0, 10), 'drone_count': (0, 10)}.get(key, (0, 9999))

def validate_value(section, key, value):
    value = value.strip()
    address = f'[{section}] {key}'
    if key not in allowed_keys(section): raise ValueError(f'{address}: unsupported setting.')
    if key == 'damage_mode':
        if value != 'real_hp': raise ValueError(f'{address}: only real_hp is supported.')
        return value
    if key in NAMES: return value
    if key in COLORS:
        if value.lower() != 'default' and not COLOR.fullmatch(value):
            raise ValueError(f'{address}: use default or RRGGBB.')
        return value
    if key in BOOLEANS:
        if value.lower() not in {'true', 'false', 'yes', 'no', '1', '0'}:
            raise ValueError(f'{address}: use true or false.')
        return value
    if key == 'level_up_sound':
        if value.lower() not in {'help_notice', 'off', 'true', 'false'}:
            raise ValueError(f'{address}: use help_notice or off.')
        return value
    if value.lower() == 'default' and ((key == 'auto_lock_range' and section.startswith('weapon.bomb_glove.level')) or (key == 'lure_range' and section.startswith('weapon.decoy_glove.level'))):
        return value
    if not NUMBER.fullmatch(value): raise ValueError(f'{address}: invalid number. Use a decimal point and no thousands separators.')
    try: number = float(value)
    except ValueError: raise ValueError(f'{address}: invalid number. Use a decimal point.') from None
    if key=='bolt_multiplier_max' and abs(number*10-round(number*10))>0.00001:
        raise ValueError(f'{address}: use steps of 0.1.')
    low, high = numeric_range(section, key)
    if not math.isfinite(number) or not low <= number <= high:
        raise ValueError(f'{address}: allowed range {low:g} … {high:g}.')
    if section == 'general' and key == 'xp_per_damage' and struct.unpack('f', struct.pack('f', number))[0] <= 0:
        raise ValueError(f'{address}: the value must be positive and representable by the game.')
    if key in INTEGER and not re.fullmatch(r'[+]?[0-9]+', value):
        raise ValueError(f'{address}: use an integer.')
    if key == 'id':
        weapon = section[7:]
        if number != WEAPONS[weapon][0]: raise ValueError(f'{address}: incorrect game ID for {weapon}.')
    return value

def validate_document(doc: IniDocument):
    for name, (item, _) in WEAPONS.items():
        section = f'weapon.{name}'
        if doc.entries.get((section, 'id')) != str(item): raise ValueError(f'Missing [{section}] id = {item}.')
        for key in XP:
            if key=='xp_v5_to_v6' and name!='wrench':continue
            if (section, key) not in doc.entries: raise ValueError(f'Missing [{section}] {key}.')
        for level in range(1, 9):
            if f'{section}.level{level}' not in doc.sections:
                raise ValueError(f'Missing [{section}.level{level}].')
    for grade in range(1,9):
        section=f'weapon.ryno.level{grade}'
        if all(doc.entries.get((section,k),'false').lower() in {'true','1','yes','on'} for k in ('continuous_missiles_enabled','shotgun_enabled')):
            raise ValueError(f'[{section}] Select either continuous missiles or shotgun mode, not both.')
    from schema import AMMO_DROP_DEFAULTS, normalized_ammo_chances
    normalized_ammo_chances({key: doc.entries.get(('general', key), default)
                             for key, default in AMMO_DROP_DEFAULTS.items()})
    for section in doc.sections: allowed_keys(section)
    for (section, key), value in doc.entries.items(): validate_value(section, key, value)
    if doc.entries.get(('general','damage_mode'),'real_hp') == 'real_hp':
        for weapon in WEAPONS:
            if weapon == 'taunter': continue
            field = 'damage_per_second' if weapon in {'pyrocitor','tesla_claw'} else 'chicken_damage_hp' if weapon == 'morph_ray' else 'damage_hp'
            for level in range(1,9):
                section=f'weapon.{weapon}.level{level}'
                if (section,field) not in doc.entries: raise ValueError(f'Missing [{section}] {field} for the real-HP damage system.')
                if any((section,k) in doc.entries for k in ('damage_multiplier','shot_damage_multiplier','chicken_damage_multiplier','explosion_damage_multiplier')):
                    raise ValueError(f'[{section}]: use absolute HP values without legacy damage multipliers.')
    return doc

def checked_document(raw):
    return validate_document(IniDocument(migrate_health_cap(raw)))

def profile_document(raw):
    from migrate_ini import convert
    return checked_document(convert(raw)[0])

def save_base_profile(kit,raw,expected=None):
    path=kit/'weapons.ini';current=path.read_bytes() if path.is_file() else None
    if expected is not None and current!=expected:
        raise ValueError('weapons.ini changed outside the launcher. Reload the default configuration before saving your editor changes.')
    clean=profile_document(raw).raw
    if current!=clean:
        if current is not None:atomic_write(kit/'backup/weapons.ini'/stamp()/'weapons.ini',current)
        atomic_write(path,clean)
    return path

def preset_name(name):
    name = name.strip()
    if (not name or len(name) > 60 or name[-1:] in {'.', ' '} or
        any(ord(c) < 32 or c in '<>:"/\\|?*' for c in name) or
        name in {'.', '..'} or name.split('.')[0].upper() in {'CON', 'PRN', 'AUX', 'NUL', *(f'COM{i}' for i in range(1, 10)), *(f'LPT{i}' for i in range(1, 10))}):
        raise ValueError('Invalid preset name. Use 1–60 characters without reserved Windows filename symbols.')
    return name

class PresetStore:
    def __init__(self, kit: Path): self.kit = kit.resolve(); self.folder = self.kit/'presets'
    def names(self):
        return sorted((p.stem for p in self.folder.glob('*.ini')), key=str.casefold)
    def path(self, name):
        name = preset_name(name)
        # Reuse spelling even on Linux to keep Windows case-insensitive behavior.
        actual = next((n for n in self.names() if n.casefold() == name.casefold()), name)
        path = self.folder/(actual + '.ini')
        if path.resolve().parent != self.folder.resolve(): raise ValueError('Preset path is outside the presets folder.')
        return path
    def read(self, name):
        return profile_document(self.path(name).read_bytes())
    def save(self, name, raw):
        raw=profile_document(raw).raw
        path = self.path(name)
        if path.exists() and path.read_bytes() != raw:
            atomic_write(self.kit/'backup/presets'/stamp()/path.name, path.read_bytes())
        atomic_write(path, raw)
        return path
    def remove(self, name):
        path = self.path(name)
        if not path.is_file(): raise ValueError('Preset not found.')
        dest = self.kit/'backup/preset_rimossi'/stamp()/path.name
        dest.parent.mkdir(parents=True, exist_ok=True); path.rename(dest)
        return dest

def bootstrap_ini(kit):
    local = kit/'weapons.ini'
    if local.exists(): return local, 'Local INI'
    candidates = [kit.parent/name/'weapons.ini' for name in PREVIOUS]
    source = next((p for p in candidates if p.is_file()), kit/'weapons.example.ini')
    checked_document(source.read_bytes())
    atomic_write(local, source.read_bytes())
    return local, str(source)

def portable_path(path: Path, kit: Path):
    path = path.resolve()
    try: return str(path.relative_to(kit.resolve()))
    except ValueError: return str(path)

def resolve_path(text, kit):
    if not text: return None
    path = Path(text).expanduser()
    return (kit/path).resolve() if not path.is_absolute() else path.resolve()

def read_settings(kit):
    path = kit/'launcher-settings.json'
    value = json.loads(path.read_text(encoding='utf-8')) if path.exists() else {}
    if not isinstance(value, dict): raise ValueError('Invalid launcher-settings.json.')
    return {k: v for k, v in value.items() if k in {'exe', 'data', 'save', 'last_preset', 'language'} and isinstance(v, str)}

def save_settings(kit, settings):
    atomic_write(kit/'launcher-settings.json', (json.dumps(settings, ensure_ascii=False, indent=2)+'\n').encode('utf-8'))

def preferred_exe(kit, stored=None):
    # Packaged mod runtime wins over author settings pointing at an old build.
    for path in [kit/'runtime/rerac.exe', kit/'rerac.exe']:
        if path.is_file(): return path.resolve()
    saved = resolve_path(stored,kit)
    return saved if saved and saved.is_file() else discover_exe(kit)

def discover_exe(kit):
    candidates = [kit/'runtime/rerac.exe', kit/'rerac.exe']
    candidates += [kit.parent/name/'runtime/rerac.exe' for name in PREVIOUS]
    return next((p.resolve() for p in candidates if p.is_file()), None)

def discover_data(kit, exe=None, env=None):
    env = os.environ if env is None else env
    explicit = env.get('RC_DATA_DIR') or env.get('RC_EXTRACTED')
    if explicit: return Path(explicit).expanduser().resolve()
    candidates = [kit/'data', kit/'extracted']
    if env.get('LOCALAPPDATA'): candidates.append(Path(env['LOCALAPPDATA'])/'rerac/games/rac1/data')
    if exe:
        info = exe.parent/'build-info.json'
        if info.is_file():
            try:
                root = json.loads(info.read_text(encoding='utf-8')).get('root')
                if root: candidates.append(Path(root)/'extracted')
            except (ValueError, OSError): pass
    return next((p.resolve() for p in candidates if valid_data(p)), None)

def valid_data(path):
    return path and (path/'toc.bin').is_file() and (path/'extract-info.json').is_file()

def probe_exe(exe, required_config=0):
    if not exe or not exe.is_file(): raise ValueError('Choose the modded rerac.exe in Launch settings. Use the runtime included with this mod.')
    flags = {'creationflags': subprocess.CREATE_NO_WINDOW} if os.name == 'nt' else {}
    with native_process_context():
        result = subprocess.run([str(exe), '--version-json'], capture_output=True, text=True,
                                encoding='utf-8', errors='replace', timeout=10, check=True, stdin=subprocess.DEVNULL, env=native_environment(os.environ), **flags)
    version = json.loads(result.stdout.strip())
    if not isinstance(version, dict) or version.get('name') != 'rerac' or version.get('game') != 'rac1' or version.get('data_format') != 1 or not version.get('version'):
        raise ValueError('Unrecognized ReRAC executable.')
    if str(version['version']).endswith('-fixed'):
        raise ValueError('This fixed-profile build ignores external configurations. Build the editable v45 game and select its runtime/rerac.exe.')
    if required_config and version.get('weapon_evolution_config', 0) < required_config:
        raise ValueError('This executable does not support the required HP and stream interval settings. Use the newly compiled runtime/rerac.exe in your playable mod folder.')
    if not (exe.parent/'assets/shaders').is_dir():
        raise ValueError('Missing assets/shaders beside rerac.exe. Select the complete mod runtime.')
    return version

def ensure_closed():
    if os.name == 'nt':
        result = subprocess.run(['tasklist', '/FI', 'IMAGENAME eq rerac.exe', '/FO', 'CSV', '/NH'],
                                capture_output=True, text=True, errors='replace', timeout=10, check=True,
                                creationflags=subprocess.CREATE_NO_WINDOW)
        if '"rerac.exe"' in result.stdout.lower(): raise ValueError('Close ReRAC before starting another session.')

def game_environment(ini, data, save=None, env=None):
    result = dict(os.environ if env is None else env)
    for key in ('RC_LEVEL', 'RC_GIVE_ITEMS', 'RC_GIVE_GOLD_WEAPONS', 'RC_WEAPON_EVO_TEST',
                'RC_WEAPON_EVO_ICON_TEST_LEVEL', 'RC_DEBUG_HIT', 'RC_PLAY_SCRIPT', 'RC_PLAY_FLY'):
        result.pop(key, None)
    if result.get('RC_SAVE_DIR') == '0': result.pop('RC_SAVE_DIR')
    if save: result['RC_SAVE_DIR'] = str(save.resolve())
    # No global environment edits; every launch receives its own full INI path.
    result.update(RC_WEAPON_EVOLUTION_INI=str(ini.resolve()), RC_COMBAT_STRAFE='1',
                  RC_BLASTER_SHOT_GATE_TICKS='2', RC_FRONTEND='1', RC_DATA_DIR=str(data.resolve()))
    if result.get('RC_WEAPON_LOCATION_TEST','').strip().lower() in {'1','true','on'}:
        result.update(RC_LEVEL='18',RC_FRONTEND='0',RC_SHIP='2')
    return native_environment(result)

@dataclass
class LaunchSession:
    process: object
    folder: Path
    ini: Path
    log: Path
    preset: str
    live_id: int = 0
    live_pending: int = 0

def live_restart_fields(before, after):
    cold = {'ngplus_multiplier','enemy_health_multiplier','enemy_health_include_bosses',
            'extra_enemies_per_level','extra_enemy_prefer_small','extra_enemy_prefer_medium',
            'extra_enemy_prefer_large','enemy_damage_by_health','nanotech_xp_per_upgrade','nanotech_max_health','nanotech_progression_max_health',
            'experimental_ammo_unlock_mode', 'golden_bolts_to_weapons',
            'golden_bolts_early_bias_percent', 'golden_bolts_one_weapon_per_planet',
            'hide_ryno', 'randomizer_mode', 'damage_mode'}
    cold.update(key for section,key in after.entries if section=='general' and key.startswith('ammo_'))
    return sorted(key for section,key in after.entries if section=='general' and key in cold
                  and after.entries[section,key]!=before.entries.get((section,key)))

def apply_live(session, raw):
    """The immutable startup INI retains restart-only mode/save policies."""
    if session.process.poll() is not None: raise ValueError('The game is no longer running.')
    if session.live_pending: raise ValueError('Wait for the previous live update to finish.')
    doc=profile_document(raw)
    deferred=live_restart_fields(IniDocument(session.ini.read_bytes()),doc)
    from migrate_ini import runtime_profile
    session.live_id+=1
    atomic_write(session.folder/f'live-{session.live_id}.ini',runtime_profile(doc.raw))
    atomic_write(session.folder/f'live-{session.live_id}.request',str(session.live_id).encode('ascii'))
    session.live_pending=session.live_id
    return deferred

def live_reply(session):
    if not session.live_pending:return None
    try:parts=(session.folder/f'live-{session.live_pending}.ack').read_text(encoding='utf-8').splitlines()
    except FileNotFoundError:return None
    if len(parts)<2 or parts[0]!=str(session.live_pending):return None
    session.live_pending=0
    return parts[1]=='ok', '\n'.join(parts[2:])

def required_config(doc):
    if doc.entries.get(('general','randomizer_mode'),'false').lower() in {'true','on','1'}:return 85
    if any(sec.startswith('weapon.wrench.level') and key=='trail_color' for sec,key in doc.entries):return 78
    if ('general','ngplus_multiplier') in doc.entries:return 77
    if any(sec.endswith(('.level6','.level7','.level8')) or key=='hide_ryno' for sec,key in doc.entries):return 76
    if ('general','nanotech_max_health') in doc.entries: return 75
    from schema import WORLD_DEFAULTS
    if any(("general",key) in doc.entries for key in WORLD_DEFAULTS):return 60
    from schema import MODEL_NEW_KEYS
    if any(key in MODEL_NEW_KEYS for _, key in doc.entries): return 58
    if any(sec.startswith("weapon.wrench.level") and key in {"hit_radius_multiplier","knockback_multiplier"} for sec,key in doc.entries): return 56
    if any(key == "experimental_max_ammo" for _,key in doc.entries): return 55
    if any(sec.startswith("weapon.wrench") for sec,_ in doc.entries): return 54
    if any(k.startswith('grenade_') or k == 'flame_mine_size_percent' or (sec.startswith('weapon.visibomb.level') and k == 'explosion_damage_radius') for sec,k in doc.entries): return 53
    from schema import EXPERIMENTAL_DEFAULTS, STANDARD_DEFAULTS
    keys = {k for fields in [*EXPERIMENTAL_DEFAULTS.values(), *STANDARD_DEFAULTS.values()] for k in fields}
    from schema import AMMO_UNLOCK_DEFAULTS, AMMO_DROP_DEFAULTS
    if any(('general', key) in doc.entries for key in (*AMMO_UNLOCK_DEFAULTS, *AMMO_DROP_DEFAULTS)): return 52
    if ('general', 'experimental_ammo_unlock_mode') in doc.entries: return 51
    if any(k == 'drone_count' for _, k in doc.entries): return 48
    if any(k in {'shotgun_horizontal_spread_degrees','shotgun_vertical_spread_degrees'} for _,k in doc.entries): return 46
    return 45 if any(k in keys for _, k in doc.entries) else 38

def launch_game(kit, raw, preset, exe, data, save=None, env=None):
    doc = profile_document(raw)
    from migrate_ini import runtime_profile
    runtime_raw = runtime_profile(doc.raw)
    launch_env=os.environ if env is None else env
    local_test=launch_env.get('RC_WEAPON_LOCATION_TEST','').strip().lower() in {'1','true','on'}
    probe_exe(exe, max(required_config(doc),86))
    if not valid_data(data): raise ValueError('Choose your extracted game data folder. It must contain toc.bin and extract-info.json.')
    ensure_closed()
    directory = kit/'runs'/stamp(); directory.mkdir(parents=True)
    ini = directory/'weapons.ini'
    # The Rust reader expects UTF-8 without BOM; user presets retain their exact BOM.
    atomic_write(ini, runtime_raw)
    log = directory/'game.log'
    metadata = {'preset': preset, 'ini': str(ini.resolve()), 'exe': str(exe.resolve()),
                'sha256': hashlib.sha256(ini.read_bytes()).hexdigest()}
    atomic_write(directory/'session.json', (json.dumps(metadata, ensure_ascii=False, indent=2)+'\n').encode('utf-8'))
    flags = {'creationflags': subprocess.CREATE_NO_WINDOW} if os.name == 'nt' else {}
    child_env=game_environment(ini, data, save, env)
    child_env['RC_WEAPON_LIVE_DIR']=str(directory.resolve())
    with log.open('wb') as stream, native_process_context():
        process = subprocess.Popen([str(exe.resolve()), '--data-dir', str(data.resolve())],
            cwd=exe.parent, env=child_env, stdout=stream, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL, **flags)
    return LaunchSession(process, directory, ini, log, preset)

def log_confirmation(text, ini):
    expected = str(ini.resolve())
    for line in text.splitlines():
        marker = 'weapon evolution: config loaded from '
        if marker in line:
            loaded = line.split(marker, 1)[1].strip()
            if os.path.normcase(loaded) == os.path.normcase(expected): return True
    return False

PLAYER_README = 'ReRAC Weapon Evolution - Windows x64\n\n1. Extract the entire ZIP into a folder.\n2. Open ReRAC_Weapon_Evolution.exe.\n3. On first launch, choose your extracted Ratchet & Clank game data folder.\n   It must contain toc.bin and extract-info.json.\n4. Press Save and launch. No preset is required.\n\nWith no named preset selected, the launcher opens weapons.ini. Edit it manually\nwhile the launcher is closed, then open the launcher and start immediately.\nChanges made in the default editor are saved to weapons.ini with a backup.\nSave as creates an optional named preset; select one explicitly to use it.\nThe next startup restores the last selected preset. Select Default configuration to return to weapons.ini.\n\nOnly the real-HP damage system is supported. There is no damage model switch.\nWeapon name opacity affects the HUD name, grade and their background only.\n100 = normal; 50 = semi-transparent; 0 = hidden. The 3D weapon is unaffected.\n\nPython and Tcl/Tk are included. Players do not need to install them.\nRust, compilation and earlier patches are not required.\nThe modded game is included in runtime; copyrighted game data is not included.\nKeep the EXE, INI files and runtime folder together when moving the mod.\nExisting game saves use the default save location unless you choose another.\nOpen the included EXE directly; do not import this ZIP into the official launcher.\n'
PLAYER_README += '\nAuthorship and rights\n'+'Weapon Evolution is an unofficial, independent mod by TomRus47. TomRus47 develops this mod only and does not develop the original Ratchet & Clank games or the ReRAC project. TomRus47 does not own the original games, characters, trademarks or game assets and is not affiliated with Sony Interactive Entertainment or Insomniac Games. Original-game rights remain with their respective rights holders. ReRAC retains its existing license and contributor credits.\n'

DISTRIBUTED_FILES = ('page_views.py','page_smoke.py','scroll_paint.py','build_lifecycle.py','i18n.py', 'icon_assets.py', 'final_release.py', 'weapons.default.ini', 'frozen_support.py', 'build_launcher_exe.py', 'windows_entry.py', 'CREA_LAUNCHER_EXE.bat', 'launcher.py', 'launcher_core.py', 'schema.py', 'Editor.bat', 'LEGGIMI.txt',
                     'CREA_PACCHETTO.bat', 'package_launcher.py', 'migrate_ini.py', 'LICENSE')

def package_distribution(kit, destination, raw, exe, include_presets=True, launcher_exe=None):
    """Bundle only mod runtime, shaders and configuration; never data or saves."""
    doc = profile_document(raw)
    version = probe_exe(exe, required_config(doc))
    if launcher_exe is not None and not launcher_exe.is_file(): raise ValueError('The compiled launcher EXE is missing.')
    destination = destination.resolve()
    if destination.exists(): raise ValueError('Choose a new output folder; the selected folder already exists.')
    if (kit/'presets').resolve() == destination or (kit/'presets').resolve() in destination.parents:
        raise ValueError('The release cannot be created inside presets.')
    if exe.parent.resolve() == destination or exe.parent.resolve() in destination.parents:
        raise ValueError('The release cannot be created inside the source runtime folder.')
    archive = Path(str(destination)+'.zip')
    if archive.exists(): raise ValueError('The output ZIP already exists. Choose a different name.')
    destination.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix='.rerac-package-', dir=destination.parent))
    try:
        if launcher_exe is not None:
            shutil.copy2(launcher_exe,stage/'ReRAC_Weapon_Evolution.exe')
            shutil.copy2(kit/'LICENSE',stage/'LICENSE')
            if (launcher_exe.parent/'licenses').is_dir():
                shutil.copytree(launcher_exe.parent/'licenses',stage/'licenses')
            (stage/'LEGGIMI.txt').write_text(PLAYER_README,encoding='utf-8')
        else:
            for name in DISTRIBUTED_FILES: shutil.copy2(kit/name, stage/name)
            instructions=stage/'LEGGIMI.txt'
            instructions.write_text('PYTHON DEVELOPMENT PACKAGE\nOpen Editor.bat.\n\n'+instructions.read_text(encoding='utf-8'),encoding='utf-8')
        icon=kit/'launcher.ico'
        if icon.is_file(): shutil.copy2(icon,stage/'launcher.ico')
        canonical = doc.raw.removeprefix(b'\xef\xbb\xbf')
        atomic_write(stage/'weapons.ini', canonical)
        atomic_write(stage/'weapons.example.ini', canonical)
        (stage/'runtime').mkdir()
        shutil.copy2(exe, stage/'runtime/rerac.exe')
        shutil.copytree(exe.parent/'assets/shaders', stage/'runtime/assets/shaders')
        atomic_write(stage/'runtime/build-info.json', (json.dumps({'mode': 'editable', 'launcher_version': VERSION,
            'version': version['version']}, indent=2)+'\n').encode('utf-8'))
        save_settings(stage, {'exe': 'runtime/rerac.exe', 'data': '', 'save': '', 'last_preset': ''})
        if include_presets:
            store = PresetStore(kit)
            for name in store.names():
                profile=profile_document(store.path(name).read_bytes())
                atomic_write(stage/'presets'/store.path(name).name,profile.raw)
        stage.rename(destination)
        try:
            with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
                for path in sorted(destination.rglob('*')):
                    if path.is_file(): z.write(path, Path(destination.name)/path.relative_to(destination))
            with zipfile.ZipFile(archive) as z:
                if z.testzip() is not None: raise ValueError('ZIP integrity check failed.')
        except Exception:
            archive.unlink(missing_ok=True)
            raise
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        atomic_write(Path(str(archive)+'.sha256'), (digest+'  '+archive.name+'\n').encode('utf-8'))
        return archive
    finally:
        if stage.exists(): shutil.rmtree(stage)


# BEGIN WEAPON EVOLUTION V64 LAVA
_required_config_before_lava_v64 = required_config

def required_config(doc):
    # Older runtimes reject these fields even with lava switched off.
    if any(section.startswith('weapon.pyrocitor.level') and key.startswith('lava_')
           for section,key in doc.entries):
        return max(64, _required_config_before_lava_v64(doc))
    return _required_config_before_lava_v64(doc)

# END WEAPON EVOLUTION V64 LAVA


# WEAPON EVOLUTION V65 LAVA ELEVATION
_required_config_before_v65 = required_config

def required_config(doc):
    if any(section.startswith('weapon.pyrocitor.level') and key=='lava_launch_angle_degrees'
           for section,key in doc.entries):
        return max(65, _required_config_before_v65(doc))
    return _required_config_before_v65(doc)

# Weapon Evolution v66: upgrade ammunition reward
_v66_previous_required_config=required_config
def required_config(doc):
    required=_v66_previous_required_config(doc)
    return max(66,required) if ('general','refill_ammo_on_upgrade') in doc.entries else required


# Weapon Evolution v67: seeded Gold Bolt rewards.
_v67_required_config=required_config
def required_config(doc):
    previous=_v67_required_config(doc)
    if ('hud','nanotech_xp_bar_opacity_percent') in doc.entries: return max(70,previous)
    return max(67,previous) if ('general','golden_bolts_to_weapons') in doc.entries else previous
_v67_numeric_range=numeric_range
def numeric_range(section,key):
    if section=='general' and key=='golden_bolts_early_bias_percent': return 0,100
    return _v67_numeric_range(section,key)

_previous_required_config_v71=required_config
def required_config(doc):
    keys={'enemy_speed_multiplier','enemy_projectile_speed_multiplier',
        'enemy_projectile_range_multiplier','golden_bolts_one_weapon_per_planet'}
    previous=_previous_required_config_v71(doc)
    return max(71,previous) if any(('general',key) in doc.entries for key in keys) else previous

_previous_required_config_v73=required_config
def required_config(doc):
    previous=_previous_required_config_v73(doc)
    return max(73,previous) if ('general','enemy_chase_range_multiplier') in doc.entries else previous


_control_required_config=required_config
def required_config(doc):
    previous=_control_required_config(doc)
    return max(82,previous) if any(('general',key) in doc.entries for key in ('dpad_quick_select','r1_experimental')) else previous

_r1_limit_required_config=required_config
def required_config(doc):
    previous=_r1_limit_required_config(doc)
    return max(86,previous) if any(key=='r1_experimental_max' for _,key in doc.entries) else previous

# The runtime must understand this key even when every grade has it Off.
_before_moving_comet_required_config = required_config
def required_config(doc):
    previous = _before_moving_comet_required_config(doc)
    return max(87, previous) if any(key == 'move_while_thrown' for section, key in doc.entries) else previous

_feedback_required_config=required_config
def required_config(doc):
    previous=_feedback_required_config(doc)
    keys={'burn_on_hit','hud_quickselect_only','slam_explosion_enabled','slam_explosion_damage_hp','slam_explosion_radius','slam_explosion_color','weapon_bar_alternative_position'}
    return max(88,previous) if any(key in keys for _,key in doc.entries) else previous

_hud_layer_required_config=required_config
def required_config(doc):
    previous=_hud_layer_required_config(doc)
    return max(89,previous) if ('hud','weapon_bar_behind_hud') in doc.entries else previous

# Alternate-only balance controls must never be sent to older parsers.
_before_experimental_overrides_required_config = required_config
def required_config(doc):
    previous = _before_experimental_overrides_required_config(doc)
    keys = {'lava_damage_bonus_percent','flame_mine_max_active_mines','flame_mine_lifetime_seconds'}
    return max(90,previous) if any(key in keys for _,key in doc.entries) else previous


# Expanded health profiles require the matching save validation and HUD packing.
_required_config_before_layered_health = required_config
def required_config(doc):
    needed=_required_config_before_layered_health(doc)
    bonus=float(doc.entries.get(('general','nanotech_max_health'),'20'))
    return max(91,needed) if bonus>64 else needed


_required_config_before_compact_nanotech = required_config
def required_config(doc):
    previous=_required_config_before_compact_nanotech(doc)
    return max(92,previous) if ('general','nanotech_progression_max_health') in doc.entries else previous
