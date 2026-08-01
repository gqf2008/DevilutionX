# -*- coding: utf-8 -*-
import re, io, csv
SRC = r'E:\Users\gxh\Documents\GitHub\DevilutionX'
rs = io.open(SRC + r'\devilutionx-rs\src\game\objdat.rs', encoding='utf-8').read()
si = rs.find('pub static ALL_OBJECTS'); ei = rs.find('];', si)
table = rs[si:ei]
blocks = []
i = table.find('ObjectData {')
while i >= 0:
    depth = 0; k = i + len('ObjectData {') - 1
    while k < len(table):
        if table[k] == '{':
            depth += 1
        elif table[k] == '}':
            depth -= 1
            if depth == 0:
                break
        k += 1
    blocks.append(table[i:k+1]); i = table.find('ObjectData {', k)
def parse(b):
    of = int(re.search(r'ofindex: (\d+)', b).group(1))
    mn = int(re.search(r'minlvl: (-?\d+)', b).group(1))
    mx = int(re.search(r'maxlvl: (-?\d+)', b).group(1))
    lt = re.search(r'olvltype: ([\w:]+)', b).group(1).split('::')[-1]
    th = re.search(r'otheme: ([\w:\d-]+)', b).group(1).split('::')[-1]
    q = re.search(r'oquest: ([\w:\d-]+)', b).group(1).split('::')[-1]
    flags = set(re.findall(r'ObjectDataFlags::(\w+)', b))
    if 'NONE' in flags and len(flags) == 1:
        flags = set()
    ad = int(re.search(r'anim_delay: (\d+)', b).group(1))
    al = int(re.search(r'anim_len: (\d+)', b).group(1))
    aw = int(re.search(r'anim_width: (\d+)', b).group(1))
    return (of, mn, mx, lt, th, q, flags, ad, al, aw)
data = [parse(b) for b in blocks]
tsv = list(csv.DictReader(io.open(SRC + r'\devilutionx-rs\tools\table_data\objdat.tsv', encoding='utf-8', newline=''), delimiter='\t'))
LT = {'DTYPE_NONE':'None','DTYPE_CATHEDRAL':'Cathedral','DTYPE_CATACOMBS':'Catacombs','DTYPE_CAVES':'Caves','DTYPE_HELL':'Hell','DTYPE_NEST':'Nest','DTYPE_CRYPT':'Crypt'}
TH = {'':'NONE','THEME_NONE':'NONE','THEME_SHRINE':'Shrine','THEME_SKELROOM':'SkelRoom','THEME_SPIDEROOM':'SpiderRoom','THEME_TREE':'Tree','THEME_CRYPT':'Crypt'}
Q = {'':'INVALID','Q_PWATER':'PWater','Q_ROCK':'Rock','Q_BLOOD':'Blood','Q_ANVIL':'Anvil','Q_SCHAMB':'Schamb','Q_BLIND':'Blind'}
# C++ ObjMasterLoadList dedup: file -> ofindex (order of first appearance)
file_idx = {}
diffs = 0
for i, (r, t) in enumerate(zip(data, tsv)):
    exp_lt = LT.get(t['levelType'] or 'DTYPE_NONE')
    exp_th = TH.get(t['theme'] or '')
    exp_q = Q.get(t['quest'] or '')
    exp_flags = set(f.strip() for f in t['flags'].split(',') if f.strip())
    exp_of = file_idx.setdefault(t['file'], len(file_idx))
    got = (r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9])
    exp = (int(t['minLevel']), int(t['maxLevel']), exp_lt, exp_th, exp_q, exp_flags,
           int(t['animDelay'] or 0), int(t['animLen'] or 0), int(t['animWidth'] or 0))
    if got != exp or r[0] != exp_of:
        diffs += 1
        if diffs <= 12:
            print('diff %d %s: rust=(of=%s,%s,%s,%s,%s,%s,%s,%s,%s,%s)' % (i, t['id'], r[0], *got))
            print('         tsv=(of=%s,%s,%s,%s,%s,%s,%s,%s,%s,%s)' % (exp_of, *exp))
print('objdat diffs:', diffs)