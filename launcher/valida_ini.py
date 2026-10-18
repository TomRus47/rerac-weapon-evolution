from pathlib import Path
import sys
from launcher_core import checked_document
from schema import NAMES

def validate(path: Path):
    doc=checked_document(path.read_bytes())
    return [f'[{s}] {k}: ignored; weapon names follow the game language' for s,k in doc.entries if k in NAMES]

if __name__=='__main__':
    try:
        validate(Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).parent/'weapons.ini')
        print('Configuration validated')
    except (ValueError,OSError,UnicodeError) as error:
        print('INI ERROR:',error,file=sys.stderr);raise SystemExit(2)
