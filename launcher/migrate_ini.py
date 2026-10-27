"""Normalize real-HP profiles without changing user balance values."""
from pathlib import Path
import re
import sys

def ensure_stream_fields(raw):
    from launcher_core import IniDocument
    doc=IniDocument(raw);newline='\r\n' if b'\r\n' in raw else '\n';lines=[]
    for line in doc.lines:
        lines.append(line)
        header=re.fullmatch(r'\s*\[([^\]]+)\]\s*',line.split(';',1)[0].strip())
        if not header:continue
        section=header[1].strip().lower()
        if not re.fullmatch(r'weapon\.(?:pyrocitor|tesla_claw)\.level[1-8]',section):continue
        additions=[key+' = '+value+newline for key,value in [('ammo_interval_ticks','10'),('damage_interval_ticks','1')] if (section,key) not in doc.entries]
        if additions:
            if not line.endswith(('\n','\r')):lines[-1]+=newline
            lines.extend(additions)
    return (b'\xef\xbb\xbf' if doc.bom else b'')+''.join(lines).encode('utf-8')

def ensure_experimental_fields(raw):
    """Order standard fields before each experimental footer, preserving balance.

    v43 geometry defaults are migrated once: 35% -> 40%, 6 -> 3 units/s,
    symmetric 33 degrees -> independent 22-degree jets. Custom values survive.
    Jump height is converted into initial upward velocity, not discarded.
    v46 renames the old R.Y.N.O. spread to horizontal spread and adds a 3-degree
    vertical default. Existing explicit axis values always take precedence.
    """
    import math
    from launcher_core import IniDocument
    from schema import EXPERIMENTAL_DEFAULTS, STANDARD_DEFAULTS, field_groups
    doc = IniDocument(raw)
    newline = '\r\n' if b'\r\n' in raw else '\n'
    blocks = []; current = []
    for line in doc.lines:
        if re.fullmatch(r'\s*\[[^\]]+\]\s*', line.split(';',1)[0].strip()):
            if current: blocks.append(current)
            current = []
        current.append(line)
    if current: blocks.append(current)
    result = []
    for block in blocks:
        header = re.fullmatch(r'\s*\[weapon\.([a-z_]+)\.level([1-8])\]\s*', block[0].split(';',1)[0].strip())
        if not header:
            result.extend(block); continue
        weapon = header[1]; section = f'weapon.{weapon}.level{header[2]}'
        if weapon not in EXPERIMENTAL_DEFAULTS and weapon not in STANDARD_DEFAULTS:
            result.extend(block); continue
        defaults = EXPERIMENTAL_DEFAULTS.get(weapon, {})
        # An R1-only footer does not enable historical experimental rewrites.
        if set(defaults)=={'r1_experimental_max'}:
            if all((section,key) in doc.entries for key in STANDARD_DEFAULTS.get(weapon,{})):
                result.extend(block);continue
            defaults={}
        values = {key: doc.entries[section,key] for sec,key in doc.entries if sec == section}
        original = dict(values)
        if 'experimental_max_ammo' in defaults:
            values.setdefault('experimental_max_ammo', values.get('capacity' if weapon == 'suck_cannon' else 'max_ammo', defaults['experimental_max_ammo']))
        legacy = set()
        if weapon == 'bomb_glove' and 'cluster_jump_height' in values:
            legacy.add('cluster_jump_height')
            values.setdefault('cluster_jump_speed', format(math.sqrt(22*float(values['cluster_jump_height'])), '.8g'))
            if values.get('cluster_bomb_size_percent') == '35': values['cluster_bomb_size_percent'] = '40'
            if values.get('cluster_launch_speed') == '6': values['cluster_launch_speed'] = '3'
        if weapon == 'pyrocitor' and 'side_flame_angle_degrees' in values:
            legacy.add('side_flame_angle_degrees')
            angle = values['side_flame_angle_degrees']
            if angle == '33': angle = '22'
            values.setdefault('side_flame_left_angle_degrees', angle)
            values.setdefault('side_flame_right_angle_degrees', angle)
        if weapon == 'ryno' and 'shotgun_spread_degrees' in values:
            # The previous single setting becomes horizontal spread. Vertical
            # gets the new restrained default unless already explicitly edited.
            legacy.add('shotgun_spread_degrees')
            values.setdefault('shotgun_horizontal_spread_degrees', values['shotgun_spread_degrees'])
            values.setdefault('shotgun_vertical_spread_degrees', '3')
        retained = []; field_lines = {}
        for line in block:
            match = re.match(r'\s*([a-zA-Z0-9_]+)\s*=', line)
            key = match[1].lower() if match else None
            if line.strip() == '; Experimental settings (on/off and options)': continue
            if key in defaults or key in legacy:
                if weapon == 'ryno' and key == 'shotgun_spread_degrees' and 'shotgun_horizontal_spread_degrees' not in original:
                    field_lines['shotgun_horizontal_spread_degrees'] = line.replace('shotgun_spread_degrees','shotgun_horizontal_spread_degrees',1)
                if key in defaults and values.get(key) == original.get(key): field_lines[key] = line
                continue
            retained.append(line)
        while retained and not retained[-1].strip(): retained.pop()
        if retained and not retained[-1].endswith(('\n','\r')): retained[-1] += newline
        for key, value in STANDARD_DEFAULTS.get(weapon, {}).items():
            if key not in values: retained.append(key+' = '+value+newline)
        if defaults:
            retained.extend([newline, '; Experimental settings (on/off and options)'+newline])
            for key, default in defaults.items():
                line = field_lines.get(key, key+' = '+values.get(key, default)+newline)
                if not line.endswith(('\n','\r')): line += newline
                retained.append(line)
        retained.append(newline)
        result.extend(retained)
    return (b'\xef\xbb\xbf' if doc.bom else b'')+''.join(result).encode('utf-8')


def ensure_global_fields(raw):
    """Preserve user values/comments; keep all mode controls after base options."""
    from launcher_core import IniDocument
    from schema import GLOBAL_EXPERIMENTAL_DEFAULTS, GLOBAL_STANDARD_DEFAULTS
    doc = IniDocument(raw)
    newline = '\r\n' if b'\r\n' in raw else '\n'
    defaults = {**GLOBAL_STANDARD_DEFAULTS, **GLOBAL_EXPERIMENTAL_DEFAULTS}
    if all(('general', key) in doc.entries for key in defaults):
        # A normalized document is already stable, including custom formatting.
        keys = doc.keys('general')
        expected = [k for k in GLOBAL_EXPERIMENTAL_DEFAULTS if k in keys]
        actual = [k for k in keys if k in GLOBAL_EXPERIMENTAL_DEFAULTS]
        if actual == expected and all(keys.index(k) < keys.index(expected[0])
                for k in keys if k not in GLOBAL_EXPERIMENTAL_DEFAULTS): return raw
    blocks = []; current = []
    for line in doc.lines:
        if re.fullmatch(r'\s*\[[^\]]+\]\s*', line.split(';',1)[0].strip()):
            if current: blocks.append(current)
            current = []
        current.append(line)
    if current: blocks.append(current)
    result = []; found = False
    for block in blocks:
        if block[0].split(';',1)[0].strip().lower() != '[general]':
            result.extend(block); continue
        found = True; mode_lines = {}; kept = []
        for line in block:
            match = re.match(r'\s*([a-zA-Z0-9_]+)\s*=', line)
            key = match[1].lower() if match else None
            if key in GLOBAL_EXPERIMENTAL_DEFAULTS: mode_lines[key] = line; continue
            if line.strip() == '; Experimental game mode': continue
            kept.append(line)
        while kept and not kept[-1].strip(): kept.pop()
        if kept and not kept[-1].endswith(('\r','\n')): kept[-1] += newline
        for key, value in GLOBAL_STANDARD_DEFAULTS.items():
            if ('general', key) not in doc.entries: kept.append(f'{key} = {value}'+newline)
        kept.extend([newline, '; Experimental game mode'+newline])
        for key, default in GLOBAL_EXPERIMENTAL_DEFAULTS.items():
            kept.append(mode_lines.get(key, f'{key} = {default}'+newline).rstrip('\r\n')+newline)
        result.extend([*kept, newline])
    if not found:
        result = ['[general]'+newline, *(f'{key} = {value}'+newline for key,value in defaults.items()), newline, *result]
    return (b'\xef\xbb\xbf' if doc.bom else b'')+''.join(result).encode('utf-8')


def convert(raw):
    """Keep absolute HP values; remove obsolete UI fields and the internal model flag."""
    from launcher_core import IniDocument
    doc=IniDocument(raw)
    if doc.entries.get(('general','damage_mode'),'real_hp')!='real_hp':
        raise ValueError('This release only supports real-HP profiles. Import an absolute-HP configuration instead of legacy damage multipliers.')
    obsolete={'wrench_gold_texture','name','name_en','name_it','name_fr','name_de','name_es','xp_bar_opacity_percent'}
    remove={index for (section,key),index in doc.addresses.items()
            if key in obsolete or (section,key)==('general','damage_mode')}
    lines=[];section=''
    for i,line in enumerate(doc.lines):
        header=re.fullmatch(r'\s*\[([^\]]+)\]\s*',line.split(';',1)[0].strip())
        if header:section=header[1].strip().lower()
        if i not in remove:lines.append(line)
    output=(b'\xef\xbb\xbf' if doc.bom else b'')+''.join(lines).encode('utf-8')
    return order_profile(ensure_hud_fields(ensure_wrench_fields(ensure_global_fields(ensure_experimental_fields(ensure_stream_fields(output))),doc.entries.get(("general","wrench_gold_texture"),"false")))),[]

def runtime_profile(raw):
    """Add the engine's real-HP flag only to the private session copy."""
    from launcher_core import IniDocument
    public,_=convert(raw);doc=IniDocument(public)
    newline='\r\n' if b'\r\n' in public else '\n'
    lines=[];found=False
    for line in doc.lines:
        lines.append(line)
        header=re.fullmatch(r'\s*\[general\]\s*',line.split(';',1)[0].strip(),re.I)
        if header:
            if not line.endswith(('\r','\n')):lines[-1]+=newline
            lines.append('damage_mode = real_hp'+newline);found=True
    if not found:lines=['[general]'+newline,'damage_mode = real_hp'+newline,newline,*lines]
    return ''.join(lines).encode('utf-8')

def import_previous_presets(kit):
    # Existing local presets remain untouched; legacy profiles are never rescaled.
    return []



def ensure_wrench_fields(raw,legacy_gold='false'):
    """Add missing track/settings without overwriting personal grade values."""
    from launcher_core import IniDocument
    doc=IniDocument(raw);nl='\r\n' if b'\r\n' in raw else '\n'
    section='weapon.wrench';blocks=[]
    defaults={'id':'8','xp_v1_to_v2':'1000','xp_v2_to_v3':'2000','xp_v3_to_v4':'3500','xp_v4_to_v5':'5500'}
    def add(section,fields):
        nonlocal raw,doc
        missing={k:v for k,v in fields.items() if (section,k) not in doc.entries}
        if not missing:return
        lines=doc.lines[:]
        if section in doc.sections:
            start=next(i for i,l in enumerate(lines) if l.split(';',1)[0].strip()=='['+section+']')
            end=next((i for i in range(start+1,len(lines)) if lines[i].lstrip().startswith('[')),len(lines))
            lines[end:end]=[k+' = '+v+nl for k,v in missing.items()]+[nl]
        else:lines += [nl+'['+section+']'+nl]+[k+' = '+v+nl for k,v in missing.items()]+[nl]
        raw=(b'\xef\xbb\xbf' if doc.bom else b'')+''.join(lines).encode();doc=IniDocument(raw)
    add(section,defaults)
    for n in range(1,6):
        add(section+f'.level{n}',{'damage_hp':str([100,125,150,175,200][n-1]),'throw_range':'10','throw_speed':'23',
            'gold_texture':legacy_gold,'show_hud':'true','crate_breaker_enabled':'false','crate_breaker_radius':'5',
            'moving_attacks_enabled':'true','bolt_attraction_bonus':'0','hit_radius_multiplier':'1','knockback_multiplier':'1'})
    return raw


def order_profile(raw):
    """Reorder sections and settings without changing values or dropping comments."""
    from launcher_core import IniDocument
    from schema import WEAPON_ORDER, WEAPONS, field_groups
    doc=IniDocument(raw); nl='\r\n' if b'\r\n' in raw else '\n'
    blocks={}; preamble=[]; current=None
    for line in doc.lines:
        if line.strip().startswith('; ') and line.strip().endswith(' - XP progression') and line.strip() in {'; '+v[1]+' - XP progression' for v in WEAPONS.values()}:continue
        h=re.fullmatch(r'\s*\[([^\]]+)\]\s*',line.split(';',1)[0].strip())
        if h: current=h[1].strip().lower();blocks[current]=[line]
        elif current is None:preamble.append(line)
        else:blocks[current].append(line)
    output=list(preamble)
    def emit(section):
        block=blocks.pop(section,None)
        if block is None:return
        match=re.fullmatch(r'weapon\.([a-z_]+)\.level[1-8]',section)
        if not match:
            while block and not block[-1].strip():block.pop()
            block.append(nl)
            if section.startswith('weapon.') and section[7:] in WEAPONS:output.extend([nl,'; '+WEAPONS[section[7:]][1]+' - XP progression'+nl])
            output.extend(block);return
        records={}; pending=[]
        for line in block[1:]:
            if not line.strip():continue
            if line.strip()=='; Experimental settings (on/off and options)':continue
            key=re.match(r'\s*([a-zA-Z0-9_]+)\s*=',line)
            if key:
                records[key[1].lower()]=pending+[line if line.endswith(('\n','\r')) else line+nl];pending=[]
            else:pending.append(line)
        standard,experimental=field_groups(match[1],records)
        output.append(block[0] if block[0].endswith(('\n','\r')) else block[0]+nl)
        for key in standard:output.extend(records[key])
        if experimental:
            output.extend([nl,'; Experimental settings (on/off and options)'+nl])
            for key in experimental:output.extend(records[key])
        output.extend(pending)
        if output and not output[-1].endswith(('\n','\r')):output[-1]+=nl
        if output and output[-1].strip():output.append(nl)
    emit('general');emit('hud')
    for weapon in WEAPON_ORDER:
        emit('weapon.'+weapon)
        for grade in range(1,9):emit(f'weapon.{weapon}.level{grade}')
    for section in list(blocks):emit(section)
    return (b'\xef\xbb\xbf' if doc.bom else b'')+''.join(output).encode('utf-8')


if __name__=='__main__':
    if len(sys.argv)!=3:raise SystemExit('Usage: py -3 migrate_ini.py source.ini destination.ini')
    source,destination=map(Path,sys.argv[1:])
    if destination.exists():raise SystemExit('The destination already exists. Choose a different filename.')
    from launcher_core import checked_document,atomic_write
    raw,_=convert(source.read_bytes());checked_document(raw);atomic_write(destination,raw)
    print('Configuration normalized:',destination)


def ensure_hud_fields(raw):
    """Add the Nanotech overlay default without rewriting existing profile values."""
    from launcher_core import IniDocument
    doc = IniDocument(raw)
    key = 'nanotech_xp_bar_opacity_percent'
    if ('hud', key) in doc.entries:
        return raw
    newline = '\r\n' if b'\r\n' in raw else '\n'
    lines = list(doc.lines)
    for index, line in enumerate(lines):
        header = re.fullmatch(r'\s*\[hud\]\s*', line.split(';', 1)[0].strip(), re.IGNORECASE)
        if header:
            if not line.endswith(('\n', '\r')):
                lines[index] += newline
            lines.insert(index + 1, key + ' = 100' + newline)
            break
    else:
        if lines and not lines[-1].endswith(('\n', '\r')):
            lines[-1] += newline
        lines.extend([newline, '[hud]' + newline, key + ' = 100' + newline])
    return (b'\xef\xbb\xbf' if doc.bom else b'') + ''.join(lines).encode('utf-8')


def upgrade_global_settings(raw):
    """Add encounter defaults and group related controls without changing any values.

    Field lines retain inline comments, spelling, BOM and newline style. Only
    recognized global/HUD rows move; weapon blocks and unrelated rows are kept.
    """
    from schema import WORLD_DEFAULTS
    defaults={'ngplus_multiplier':'1','enemy_chase_range_multiplier':'1','enemy_speed_multiplier':'1','enemy_projectile_range_multiplier':'1',
        'enemy_projectile_speed_multiplier':'1','golden_bolts_one_weapon_per_planet':'false'}
    text=raw.decode('utf-8-sig');newline='\r\n' if '\r\n' in text else '\n'
    blocks=[];section='';lines=[]
    for line in text.splitlines(keepends=True):
        match=re.fullmatch(r'\s*\[([^]]+)\]\s*(?:[;#].*)?',line.strip())
        if match:
            if lines:blocks.append((section,lines))
            section=match[1].strip().lower();lines=[]
        lines.append(line)
    if lines:blocks.append((section,lines))
    if not any(section=='general' for section,_ in blocks):blocks.insert(0,('general',['[general]'+newline]))
    groups={'general':[list(WORLD_DEFAULTS),['golden_bolts_to_weapons','golden_bolts_early_bias_percent','golden_bolts_one_weapon_per_planet']],
        'hud':[['name_opacity_percent','weapon_bar_alternative_position','weapon_bar_behind_hud','nanotech_xp_bar_opacity_percent','show_for_weapons_without_ammo','xp_bar_height_px','xp_bar_horizontal_inset_px','level_up_popup','level_up_popup_duration_seconds','level_up_sound']]}
    result=[]
    for section,lines in blocks:
        if section=='general':
            present={m[1].lower() for line in lines if (m:=re.match(r'\s*([\w]+)\s*=',line))}
            if lines and not lines[-1].endswith(('\r','\n')):lines[-1]+=newline
            lines=lines+[key+' = '+value+newline for key,value in defaults.items() if key not in present]
        for keys in groups.get(section,[]):
            picked={};retained=[];at=None
            for line in lines:
                match=re.match(r'\s*([\w]+)\s*=',line);key=match[1].lower() if match else None
                if key in keys:
                    if at is None:at=len(retained)
                    picked[key]=line.rstrip('\r\n')+newline
                else:retained.append(line)
            if at is not None:retained[at:at]=[picked[k] for k in keys if k in picked]
            lines=retained
        result.extend(lines)
    return (b'\xef\xbb\xbf' if raw.startswith(b'\xef\xbb\xbf') else b'')+''.join(result).encode('utf-8')

_previous_convert_v71=convert
def convert(raw):
    result,warnings=_previous_convert_v71(raw)
    return upgrade_global_settings(result),warnings



def ensure_eight_grades(raw):
    """Add V6-V8 once; keep every existing field/value, BOM and line ending."""
    from launcher_core import IniDocument
    from schema import WEAPONS
    doc=IniDocument(raw);nl='\r\n' if b'\r\n' in raw else '\n'
    blocks={};prefix=[];current=None
    for line in doc.lines:
        header=re.fullmatch(r'\s*\[([^]]+)\]\s*',line.split(';',1)[0].strip())
        if header:current=header[1].strip().lower();blocks[current]=[line]
        elif current is None:prefix.append(line)
        else:blocks[current].append(line)
    def add(section,fields):
        lines=blocks.setdefault(section,['['+section+']'+nl])
        if lines and not lines[-1].endswith(('\r','\n')):lines[-1]+=nl
        lines.extend(key+' = '+value+nl for key,value in fields.items() if (section,key) not in doc.entries)
    if ('general','hide_ryno') not in doc.entries:add('general',{'hide_ryno':'false'})
    damage={'damage_hp','damage_per_second','chicken_damage_hp','grenade_damage_hp','shotgun_damage_hp','mine_flame_damage_per_second','sand_mouse_shot_damage_hp'}
    for weapon in WEAPONS:
        section='weapon.'+weapon
        last=doc.entries.get((section,'xp_v4_to_v5'),'5500')
        try:threshold=float(last)
        except ValueError:threshold=5500
        add(section,{key:format(min(1_000_000,threshold*factor),'.8g') for key,factor in
            ([('xp_v5_to_v6',1.25)] if weapon=='wrench' else [])+[('xp_v6_to_v7',1.5),('xp_v7_to_v8',2)]})
        source=section+'.level5'
        if source not in blocks:continue
        for grade,factor in [(6,1.25),(7,1.5),(8,1.75)]:
            target=section+f'.level{grade}'
            if target in blocks:continue
            lines=['['+target+']'+nl]
            for line in blocks[source][1:]:
                assignment=re.match(r'(\s*([a-zA-Z0-9_]+)\s*=\s*)([^;\r\n]*)(.*)',line.rstrip('\r\n'))
                if assignment and assignment[2] in damage:
                    try:value=format(min(1_000_000,float(assignment[3].strip())*factor),'.8g')
                    except ValueError:value=assignment[3]
                    line=assignment[1]+value+assignment[4]+nl
                elif assignment and assignment[2] in {'gold_texture','gold_effects'}:line=assignment[1]+'true'+assignment[4]+nl
                lines.append(line if line.endswith(('\r','\n')) else line+nl)
            if lines[-1].strip():lines.append(nl)
            blocks[target]=lines
    text=''.join(prefix)+''.join(''.join(lines) for lines in blocks.values())
    result=(b'\xef\xbb\xbf' if doc.bom else b'')+text.encode('utf-8')
    return order_profile(result)

_previous_convert_v76=convert
def convert(raw):
    result,warnings=_previous_convert_v76(raw)
    return ensure_eight_grades(result),warnings


def ensure_wrench_trail_color(raw):
    """Append a missing default to every existing grade; retain custom colors."""
    from launcher_core import IniDocument
    doc=IniDocument(raw);nl='\r\n' if b'\r\n' in raw else '\n'
    result=[];block=[];section=''
    def flush():
        if re.fullmatch(r'weapon\.wrench\.level[1-8]',section) and (section,'trail_color') not in doc.entries:
            if block and not block[-1].endswith(('\r','\n')):block[-1]+=nl
            block.append('trail_color = default'+nl)
        result.extend(block)
    for line in doc.lines:
        header=re.fullmatch(r'\s*\[([^]]+)\]\s*',line.split(';',1)[0].strip())
        if header:
            flush();block=[];section=header[1].strip().lower()
        block.append(line)
    flush()
    return order_profile((b'\xef\xbb\xbf' if doc.bom else b'')+''.join(result).encode())

_previous_convert_v78=convert
def convert(raw):
    result,warnings=_previous_convert_v78(raw)
    return ensure_wrench_trail_color(result),warnings


def ensure_control_fields(raw):
    """Append missing control toggles, preserving current balance and comments."""
    from launcher_core import IniDocument
    from schema import CONTROL_DEFAULTS
    doc=IniDocument(raw);nl='\r\n' if b'\r\n' in raw else '\n'
    missing={key:value for key,value in CONTROL_DEFAULTS.items() if ('general',key) not in doc.entries}
    if not missing:return raw
    lines=[];inside=False;found=False
    def append_missing():
        if lines and not lines[-1].endswith(('\n','\r')):lines[-1]+=nl
        lines.extend(key+' = '+value+nl for key,value in missing.items())
    for line in doc.lines:
        header=re.fullmatch(r'\s*\[([^]]+)\]\s*',line.split(';',1)[0].strip())
        if header:
            if inside:append_missing()
            inside=header[1].strip().lower()=='general';found|=inside
        lines.append(line)
    if inside:append_missing()
    if not found:lines=['[general]'+nl,*(key+' = '+value+nl for key,value in missing.items()),nl,*lines]
    return (b'\xef\xbb\xbf' if doc.bom else b'')+''.join(lines).encode('utf-8')

_control_convert=convert
def convert(raw):
    result,warnings=_control_convert(raw)
    return ensure_control_fields(result),warnings

def ensure_r1_limit_fields(raw):
    """Add only missing grade gates; retain every existing value and comment."""
    from launcher_core import IniDocument
    from schema import WEAPONS, EXPERIMENTAL_DEFAULTS, EXPERIMENTAL_ENABLERS, R1_MAX_KEY, R1_MODE_MAX
    doc=IniDocument(raw);nl='\r\n' if b'\r\n' in raw else '\n'
    blocks=[];block=[]
    for line in doc.lines:
        if re.fullmatch(r'\s*\[[^]]+\]\s*',line.split(';',1)[0].strip()) and block:
            blocks.append(block);block=[]
        block.append(line)
    if block:blocks.append(block)
    output=[]
    for block in blocks:
        header=re.fullmatch(r'\s*\[weapon\.([a-z_]+)\.level([1-8])\]\s*',block[0].split(';',1)[0].strip())
        if not header or header[1] not in WEAPONS:
            output.extend(block);continue
        weapon=header[1];section=f'weapon.{weapon}.level{header[2]}'
        if (section,R1_MAX_KEY) in doc.entries:
            output.extend(block);continue
        keys=[]
        for i,line in enumerate(block):
            match=re.match(r'\s*([a-zA-Z0-9_]+)\s*=',line)
            if match:keys.append((i,match[1].lower()))
        enablers=[i for i,key in keys if key in EXPERIMENTAL_ENABLERS.get(weapon,())]
        options=[i for i,key in keys if key in EXPERIMENTAL_DEFAULTS.get(weapon,{})]
        end=len(block)
        while end>0 and not block[end-1].strip():end-=1
        at=max(enablers)+1 if enablers else min(options) if options else max((i for i,_ in keys),default=end-1)+1
        if at>0 and not block[at-1].endswith(('\n','\r')):block[at-1]+=nl
        additions=[] if enablers or options else [nl,'; Experimental settings (on/off and options)'+nl]
        additions.append(R1_MAX_KEY+' = '+str(R1_MODE_MAX.get(weapon,0))+nl)
        output.extend(block[:at]+additions+block[at:])
    return (b'\xef\xbb\xbf' if doc.bom else b'')+''.join(output).encode('utf-8')

_r1_limit_convert=convert
def convert(raw):
    result,warnings=_r1_limit_convert(raw)
    return ensure_r1_limit_fields(result),warnings

def ensure_reward_multiplier_fields(raw):
    """Group only the three Randomizer/reward rows, preserving their values."""
    from launcher_core import IniDocument
    doc=IniDocument(raw);nl='\r\n' if b'\r\n' in raw else '\n'
    keys=('randomizer_mode','randomizer_bolt_multiplier','bolt_multiplier_max')
    defaults=('false','false','3');found={};lines=[];inside=False;insert=None
    for line in doc.lines:
        header=re.fullmatch(r'\s*\[([^]]+)\]\s*',line.split(';',1)[0].strip())
        if header:
            inside=header[1].strip().lower()=='general'
            if inside:insert=len(lines)+1
        match=re.match(r'\s*([a-zA-Z0-9_]+)\s*=',line) if inside else None
        if match and match[1].lower() in keys:
            key=match[1].lower();found[key]=line
            if key==keys[0]:insert=len(lines)
        else:lines.append(line)
    rows=[found.get(key,key+' = '+value+nl) for key,value in zip(keys,defaults)]
    if insert is None:lines=['[general]'+nl,*rows,nl,*lines]
    else:lines[insert:insert]=rows
    return (b'\xef\xbb\xbf' if doc.bom else b'')+''.join(lines).encode('utf-8')
_reward_multiplier_convert=convert
def convert(raw):
    result,warnings=_reward_multiplier_convert(raw)
    return ensure_reward_multiplier_fields(result),warnings

_feedback_previous_convert=convert
def convert(raw):
    result,warnings=_feedback_previous_convert(raw)
    result=order_profile(ensure_experimental_fields(result))
    from launcher_core import IniDocument
    doc=IniDocument(result)
    if ('hud','weapon_bar_alternative_position') not in doc.entries:
        nl='\r\n' if b'\r\n' in result else '\n'
        if 'hud' in doc.sections:
            lines=list(doc.lines)
            at=next(i for i,line in enumerate(lines) if re.match(r'\s*\[hud\]',line,re.I))
            if not lines[at].endswith(('\n','\r')):lines[at]+=nl
            lines.insert(at+1,'weapon_bar_alternative_position = false'+nl)
            result=(b'\xef\xbb\xbf' if doc.bom else b'')+''.join(lines).encode('utf-8')
        else:result+= (nl+'[hud]'+nl+'weapon_bar_alternative_position = false'+nl).encode()
    return result,warnings

_hud_layer_previous_convert=convert
def convert(raw):
    result,warnings=_hud_layer_previous_convert(raw)
    from launcher_core import IniDocument
    doc=IniDocument(result)
    if ('hud','weapon_bar_behind_hud') not in doc.entries:
        nl='\r\n' if b'\r\n' in result else '\n'
        lines=list(doc.lines)
        at=next(i for i,line in enumerate(lines) if re.match(r'\s*\[hud\]',line,re.I))
        if not lines[at].endswith(('\n','\r')):lines[at]+=nl
        lines.insert(at+1,'weapon_bar_behind_hud = true'+nl)
        result=(b'\xef\xbb\xbf' if doc.bom else b'')+''.join(lines).encode('utf-8')
    return order_profile(upgrade_global_settings(result)),warnings
