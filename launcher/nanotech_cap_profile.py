"""Migrate the old XP-only cap to a base-plus-XP cap, preserving other bytes.

The distinct key makes this conversion idempotent and keeps old runtime INIs
readable. Four starting HP belong to progression; Premium/Ultra reserve four more.
"""
import re
from decimal import Decimal

LEGACY_KEY = 'nanotech_max_health'
KEY = 'nanotech_progression_max_health'
MINIMUM = 4
MAXIMUM = 206


def migrate_health_cap(raw):
    bom = raw.startswith(b'\xef\xbb\xbf')
    lines = raw.decode('utf-8-sig').splitlines(keepends=True)
    section = ''
    found = {LEGACY_KEY: [], KEY: []}
    assignment = re.compile(r'^(\s*)([A-Za-z0-9_]+)(\s*=\s*)([^;#\r\n]*)(.*)$')
    for index, line in enumerate(lines):
        code = line.split(';', 1)[0].split('#', 1)[0].strip()
        header = re.fullmatch(r'\[([^]]+)\]', code)
        if header:
            section = header[1].strip().lower()
            continue
        if section != 'general':
            continue
        match = assignment.match(line.rstrip('\r\n'))
        if match and match[2].lower() in found:
            found[match[2].lower()].append((index, match))
    if any(len(entries) > 1 for entries in found.values()):
        raise ValueError('Duplicate Nanotech health-cap setting in [general].')
    if not found[LEGACY_KEY]:
        return raw
    index, match = found[LEGACY_KEY][0]
    if found[KEY]:
        # Explicit canonical setting wins; the retired key cannot shadow it.
        del lines[index]
    else:
        value = match[4].strip()
        if not re.fullmatch(r'[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?', value):
            raise ValueError('Legacy nanotech_max_health must be an integer 0..300.')
        number = Decimal(value)
        if not number.is_finite() or not 0 <= number <= 300 or number != number.to_integral_value():
            raise ValueError('Legacy nanotech_max_health must be an integer 0..300.')
        converted = min(MAXIMUM, int(number) + MINIMUM)
        ending = lines[index][len(lines[index].rstrip('\r\n')):]
        trailing = match[4][len(match[4].rstrip()):]
        lines[index] = match[1] + KEY + match[3] + str(converted) + trailing + match[5] + ending
    return (b'\xef\xbb\xbf' if bom else b'') + ''.join(lines).encode('utf-8')
