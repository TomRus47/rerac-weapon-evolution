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

def convert(raw):
    """Keep absolute HP values; remove obsolete UI fields and the internal model flag."""
    from launcher_core import IniDocument
    doc=IniDocument(raw)
    if doc.entries.get(('general','damage_mode'),'real_hp')!='real_hp':
        raise ValueError('This release only supports real-HP profiles. Import an absolute-HP configuration instead of legacy damage multipliers.')
    obsolete={'name','name_en','name_it','name_fr','name_de','name_es','xp_bar_opacity_percent'}
    remove={index for (section,key),index in doc.addresses.items()
            if key in obsolete or (section,key)==('general','damage_mode')}
    lines=[];section=''
    for i,line in enumerate(doc.lines):
        header=re.fullmatch(r'\s*\[([^\]]+)\]\s*',line.split(';',1)[0].strip())
        if header:section=header[1].strip().lower()
        if section=='weapon.wrench' or section.startswith('weapon.wrench.level'):continue
        if i not in remove:lines.append(line)
    output=(b'\xef\xbb\xbf' if doc.bom else b'')+''.join(lines).encode('utf-8')
    return ensure_stream_fields(output),[]

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

if __name__=='__main__':
    if len(sys.argv)!=3:raise SystemExit('Usage: py -3 migrate_ini.py source.ini destination.ini')
    source,destination=map(Path,sys.argv[1:])
    if destination.exists():raise SystemExit('The destination already exists. Choose a different filename.')
    from launcher_core import checked_document,atomic_write
    raw,_=convert(source.read_bytes());checked_document(raw);atomic_write(destination,raw)
    print('Configuration normalized:',destination)
