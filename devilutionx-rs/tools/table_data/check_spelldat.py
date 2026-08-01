# -*- coding: utf-8 -*-
import re, io, csv
SRC = r'E:\Users\gxh\Documents\GitHub\DevilutionX'
rs = io.open(SRC + r'\devilutionx-rs\src\game\spelldat.rs', encoding='utf-8').read()
si = rs.find('pub static SPELLS_DATA'); ei = rs.find('];', si)
table = rs[si:ei]
blocks = []
i = table.find('SpellData {')
while i >= 0:
    depth = 0; k = i + len('SpellData {') - 1
    while k < len(table):
        if table[k] == '{':
            depth += 1
        elif table[k] == '}':
            depth -= 1
            if depth == 0:
                break
        k += 1
    blocks.append(table[i:k+1]); i = table.find('SpellData {', k)
def parse(b):
    m = re.search(r'name: "([^"]*)"', b)
    name = m.group(1) if m else ''
    bc = int(re.search(r'book_cost_10: (-?\d+)', b).group(1))
    sc = int(re.search(r'staff_cost_10: (-?\d+)', b).group(1))
    mc = int(re.search(r'mana_cost: (-?\d+)', b).group(1))
    bl = int(re.search(r'book_level: (-?\d+)', b).group(1))
    sl = int(re.search(r'staff_level: (-?\d+)', b).group(1))
    mi = int(re.search(r'min_int: (-?\d+)', b).group(1))
    ma = int(re.search(r'mana_adj: (-?\d+)', b).group(1))
    mn = int(re.search(r'min_mana: (-?\d+)', b).group(1))
    smin = int(re.search(r'staff_min: (-?\d+)', b).group(1))
    smax = int(re.search(r'staff_max: (-?\d+)', b).group(1))
    miss = re.search(r'missiles: \[([^\]]*)\]', b).group(1)
    return (name, bc, sc, mc, bl, sl, mi, ma, mn, smin, smax, miss)
data = [parse(b) for b in blocks]
base = list(csv.DictReader(io.open(SRC + r'\devilutionx-rs\tools\table_data\spelldat.tsv', encoding='utf-8', newline=''), delimiter='\t'))
hf = list(csv.DictReader(io.open(SRC + r'\devilutionx-rs\tools\table_data\spelldat_hf.tsv', encoding='utf-8', newline=''), delimiter='\t'))
tsv = {r['id']: r for r in base + hf}
diffs = 0
for i, (name, bc, sc, mc, bl, sl, mi, ma, mn, smin, smax, miss) in enumerate(data):
    if i == 0:
        continue
    key = name
    if key not in tsv:
        cand = [k for k in tsv if k.lower().replace(' ','') == name.lower().replace(' ','')]
        if not cand:
            diffs += 1; print('not found', i, repr(name)); continue
        key = cand[0]
    r = tsv[key]
    exp = (int(r['bookCost10']), int(r['staffCost10']), int(r['manaCost']), int(r['bookLevel']),
           int(r['staffLevel']), int(r['minIntelligence']), int(r['manaMultiplier']), int(r['minMana']),
           int(r['staffMin']), int(r['staffMax']))
    got = (bc, sc, mc, bl, sl, mi, ma, mn, smin, smax)
    if got != exp:
        diffs += 1
        if diffs <= 10:
            print('diff', key, got, exp)
    em = tuple(p for p in r['missiles'].split(',') if p)
    gm = tuple(x.replace('MissileID::','') for x in miss.split(',') if x.strip() and 'Null' not in x)
    if em != gm:
        diffs += 1
        if diffs <= 10:
            print('missile diff', key, gm, em)
print('spelldat real diffs:', diffs)