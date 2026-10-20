"""Preserve launcher icons and recover ICO images from previous Windows builds.

No Windows API or optional PE package is needed. Only RT_GROUP_ICON/RT_ICON
resources are copied; executable code is never imported or run.
"""
from pathlib import Path
import struct


def validate_ico(data):
    if len(data) < 6:
        raise ValueError('Incomplete ICO header.')
    reserved, kind, count = struct.unpack_from('<HHH', data)
    end = 6 + count * 16
    if reserved or kind != 1 or not 1 <= count <= 256 or end > len(data):
        raise ValueError('Invalid ICO directory.')
    for i in range(count):
        size, offset = struct.unpack_from('<II', data, 6 + i * 16 + 8)
        if size == 0 or offset < end or offset + size > len(data):
            raise ValueError('Incomplete ICO image.')
    return data


def extract_ico(path):
    data = Path(path).read_bytes()
    def get(fmt, at):
        if at < 0 or at + struct.calcsize(fmt) > len(data):
            raise ValueError('PE resource points outside the file.')
        return struct.unpack_from(fmt, data, at)
    if data[:2] != b'MZ':
        raise ValueError('Not a Windows executable.')
    pe, = get('<I', 60)
    if data[pe:pe+4] != b'PE\0\0':
        raise ValueError('Missing PE signature.')
    sections, = get('<H', pe+6)
    optional_size, = get('<H', pe+20)
    opt = pe+24
    magic, = get('<H', opt)
    directory = {0x10b: 96, 0x20b: 112}.get(magic)
    if directory is None or optional_size < directory+24 or sections > 128:
        raise ValueError('Unsupported PE optional header.')
    root_rva, root_size = get('<II', opt+directory+16)
    def rva(addr, length):
        for i in range(sections):
            _, virtual, raw_size, raw = get('<IIII', opt+optional_size+i*40+8)
            if virtual <= addr and addr-virtual+length <= raw_size and raw+addr-virtual+length <= len(data):
                return raw+addr-virtual
        raise ValueError('Unmapped PE resource.')
    root = rva(root_rva, root_size)
    def entries(offset):
        if offset < 0 or offset+16 > root_size:
            raise ValueError('Invalid PE resource directory.')
        named, count = get('<HH', root+offset+12)
        total = named+count
        if total > 4096 or offset+16+total*8 > root_size:
            raise ValueError('Invalid PE resource count.')
        return dict(get('<II', root+offset+16+i*8) for i in range(total))
    def subtree(parent, key):
        value = entries(parent).get(key)
        if value is None or not value & 0x80000000:
            raise ValueError('Required icon resource is missing.')
        return value & 0x7fffffff
    def leaf(parent):
        for _, offset in sorted(entries(parent).items()):
            if offset & 0x80000000:
                continue
            if offset+16 > root_size:
                raise ValueError('Invalid icon data entry.')
            addr, size, _, _ = get('<IIII', root+offset)
            if size > 16*1024*1024:
                raise ValueError('Icon image is too large.')
            at = rva(addr, size)
            return data[at:at+size]
        raise ValueError('No icon resource language.')
    groups = subtree(0, 14)
    icons = subtree(0, 3)
    group_ids=sorted(entries(groups))
    if not group_ids:
        raise ValueError('No icon group images.')
    group_id=group_ids[0]
    group = leaf(subtree(groups, group_id))
    if len(group) < 6:
        raise ValueError('Incomplete group icon.')
    reserved, kind, count = struct.unpack_from('<HHH', group)
    if reserved or kind != 1 or not 1 <= count <= 256 or len(group) < 6+count*14:
        raise ValueError('Invalid group icon.')
    directory_bytes = bytearray(struct.pack('<HHH', 0, 1, count))
    images = bytearray()
    for i in range(count):
        entry = group[6+i*14:6+(i+1)*14]
        size, resource_id = struct.unpack_from('<IH', entry, 8)
        image = leaf(subtree(icons, resource_id))
        if len(image) != size:
            raise ValueError('Group icon image size mismatch.')
        directory_bytes += entry[:12] + struct.pack('<I', 6+count*16+len(images))
        images += image
    return validate_ico(bytes(directory_bytes+images))


def preserve_icon(kit):
    """Keep an explicit icon; otherwise prefer pre-update launcher backups."""
    kit = Path(kit)
    target = kit/'launcher.ico'
    if target.is_file():
        validate_ico(target.read_bytes())
        return target
    candidates = sorted(kit.glob('ReRAC_Weapon_Evolution.before-v*.exe'),
                        key=lambda p: p.stat().st_mtime, reverse=True)
    candidates += [kit/'ReRAC_Weapon_Evolution.exe']
    for candidate in candidates:
        if not candidate.is_file():
            continue
        try:
            raw = extract_ico(candidate)
        except (OSError, ValueError, struct.error):
            continue
        # Exclusive creation prevents overwriting an icon selected in parallel.
        try:
            with target.open('xb') as stream:
                stream.write(raw)
        except FileExistsError:
            validate_ico(target.read_bytes())
        print('Preserved launcher icon from: '+str(candidate), flush=True)
        return target
    return None
