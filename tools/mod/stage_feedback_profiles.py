"""Stage current personal profiles with additive v102 controls; never change balance."""
from pathlib import Path
import json,sys
launcher,playable,source,stage=map(Path,sys.argv[1:5]);sys.path.insert(0,str(launcher))
import launcher_core as core
from migrate_ini import convert,runtime_profile
stage.mkdir(parents=True,exist_ok=True)
paths=[playable/'weapons.ini',*sorted((playable/'presets').glob('*.ini'))]
paths += [p for p in [launcher/'weapons.default.ini',launcher/'weapons.example.ini',source/'crates/rc-game/src/weapon_evo_release.ini'] if p.is_file()]
rows=[]
for index,path in enumerate(dict.fromkeys(paths)):
    raw=path.read_bytes();before=core.IniDocument(raw)
    new=runtime_profile(raw) if path.name=='weapon_evo_release.ini' else convert(raw)[0]
    after=core.checked_document(new)
    for pair,value in before.entries.items():
        if after.entries.get(pair)!=value:raise ValueError('An existing value would change: '+str(path)+' '+str(pair))
    if new!=raw:
        saved=stage/(str(index)+'.ini');saved.write_bytes(new)
        rows.append({'target':str(path),'staged':str(saved)})
(stage/'profiles.json').write_text(json.dumps(rows,indent=2))
