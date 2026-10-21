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
        if not re.fullmatch(r'weapon\.(?:pyrocitor|tesla_claw)\.level[1-5]',section):continue
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
        header = re.fullmatch(r'\s*\[weapon\.([a-z_]+)\.level([1-5])\]\s*', block[0].split(';',1)[0].strip())
        if not header:
            result.extend(block); continue
        weapon = header[1]; section = f'weapon.{weapon}.level{header[2]}'
        if weapon not in EXPERIMENTAL_DEFAULTS and weapon not in STANDARD_DEFAULTS:
            result.extend(block); continue
        defaults = EXPERIMENTAL_DEFAULTS.get(weapon, {})
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
    return order_profile(ensure_wrench_fields(ensure_global_fields(ensure_experimental_fields(ensure_stream_fields(output))),doc.entries.get(("general","wrench_gold_texture"),"false"))),[]

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
        match=re.fullmatch(r'weapon\.([a-z_]+)\.level[1-5]',section)
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
        for grade in range(1,6):emit(f'weapon.{weapon}.level{grade}')
    for section in list(blocks):emit(section)
    return (b'\xef\xbb\xbf' if doc.bom else b'')+''.join(output).encode('utf-8')


if __name__=='__main__':
    if len(sys.argv)!=3:raise SystemExit('Usage: py -3 migrate_ini.py source.ini destination.ini')
    source,destination=map(Path,sys.argv[1:])
    if destination.exists():raise SystemExit('The destination already exists. Choose a different filename.')
    from launcher_core import checked_document,atomic_write
    raw,_=convert(source.read_bytes());checked_document(raw);atomic_write(destination,raw)
    print('Configuration normalized:',destination)
