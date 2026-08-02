# -*- coding: utf-8 -*-
import re, io, csv, os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
RS = os.path.join(ROOT, 'src', 'game')
TSV = os.path.join(HERE)
failures = 0
def tsv_rows(name):
    return list(csv.DictReader(io.open(os.path.join(TSV, name), encoding='utf-8', newline=''), delimiter='\t'))
def rust_source(name):
    return io.open(os.path.join(RS, name), encoding='utf-8').read()

def check_monstdat():
    global failures
    rs = rust_source('monstdat.rs'); tsv = tsv_rows('monstdat_hf.tsv')
    entries = []
    i = 0
    while True:
        j = rs.find('mdat!(', i)
        if j < 0:
            break
        depth = 0; k = j + 5
        while k < len(rs):
            if rs[k] == '(':
                depth += 1
            elif rs[k] == ')':
                depth -= 1
                if depth == 0:
                    break
            k += 1
        entries.append(rs[j+6:k]); i = k + 1
    RES = {'NONE': 0, 'RESIST_MAGIC': 1<<0, 'RESIST_FIRE': 1<<1, 'RESIST_LIGHTNING': 1<<2,
           'IMMUNE_MAGIC': 1<<3, 'IMMUNE_FIRE': 1<<4, 'IMMUNE_LIGHTNING': 1<<5, 'IMMUNE_ACID': 1<<7}
    def res_mask(s):
        if not s.strip():
            return 0
        m = 0
        for part in re.split(r'[|,]', s):
            part = part.strip()
            if not part:
                continue
            names = re.findall(r'([A-Z_]+)\.0', part) or [part.rsplit('::', 1)[-1]]
            for nm in names:
                m |= RES[nm]
        return m
    AI = {'Zombie':0,'Fat':1,'SkeletonMelee':2,'SkeletonRanged':3,'Scavenger':4,'Rhino':5,'GoatMelee':6,
     'GoatRanged':7,'Fallen':8,'Magma':9,'SkeletonKing':10,'Bat':11,'Gargoyle':12,'Butcher':13,'Succubus':14,
     'Sneak':15,'Storm':16,'FireMan':17,'Gharbad':18,'Acid':19,'AcidUnique':20,'Golem':21,'Zhar':22,'Snotspill':23,
     'Snake':24,'Counselor':25,'Mega':26,'Diablo':27,'Lazarus':28,'LazarusSuccubus':29,'Lachdanan':30,'Warlord':31,
     'FireBat':32,'Torchant':33,'HorkDemon':34,'Lich':35,'ArchLich':36,'Psychorb':37,'Necromorb':38,'BoneDemon':39}
    CLS = {'Undead':0,'Demon':1,'Animal':2}
    diffs = 0
    for entry, row in zip(entries, tsv):
        args = [a.strip() for a in entry.split(',')]
        def iv(a):
            return int(a) if a.lstrip('-').isdigit() else None
        got = (args[0].strip('"'), iv(args[1]), iv(args[2]), iv(args[3]), iv(args[4]), iv(args[5]),
               AI[args[6].split('::')[1]], iv(args[7]), iv(args[8]), iv(args[9]), iv(args[10]),
               iv(args[11]), iv(args[12]), iv(args[13]), iv(args[14]), CLS[args[15].split('::')[1]],
               res_mask(args[16]), res_mask(args[17]), iv(args[18]))
        exp = (row['name'], int(row['minDunLvl']), int(row['maxDunLvl']), int(row['level']),
               int(row['hitPointsMinimum']), int(row['hitPointsMaximum']), AI[row['ai']],
               int(row['intelligence']), int(row['toHit']), int(row['minDamage']), int(row['maxDamage']),
               int(row['toHitSpecial']), int(row['minDamageSpecial']), int(row['maxDamageSpecial']),
               int(row['armorClass']), CLS[row['monsterClass']], res_mask(row['resistance']),
               res_mask(row['resistanceHell']), int(row['exp']))
        if got != exp:
            diffs += 1
    print('monstdat: %d/%d rows, %d diffs' % (len(entries), len(tsv), diffs))
    failures += diffs

def check_spelldat():
    global failures
    rs = rust_source('spelldat.rs')
    tsv = {r['id']: r for r in tsv_rows('spelldat.tsv') + tsv_rows('spelldat_hf.tsv')}
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
    def num(b, field):
        return int(re.search(field + r': (-?\d+)', b).group(1))
    def norm(s):
        return s.lower().replace(' ', '').replace('the', '')
    diffs = 0
    for bi, b in enumerate(blocks):
        if bi == 0:
            continue
        m = re.search(r'name: "([^"]*)"', b)
        name = m.group(1) if m else ''
        key = next((k for k in tsv if norm(k) == norm(name)), None)
        if key is None:
            diffs += 1
            continue
        r = tsv[key]
        got = (num(b,'book_cost_10'), num(b,'staff_cost_10'), num(b,'mana_cost'), num(b,'book_level'),
               num(b,'staff_level'), num(b,'min_int'), num(b,'mana_adj'), num(b,'min_mana'),
               num(b,'staff_min'), num(b,'staff_max'))
        exp = (int(r['bookCost10']), int(r['staffCost10']), int(r['manaCost']), int(r['bookLevel']),
               int(r['staffLevel']), int(r['minIntelligence']), int(r['manaMultiplier']), int(r['minMana']),
               int(r['staffMin']), int(r['staffMax']))
        if got != exp:
            diffs += 1
    print('spelldat: %d rows, %d diffs' % (len(blocks), diffs))
    failures += diffs

def check_objdat():
    global failures
    rs = rust_source('objdat.rs'); tsv = tsv_rows('objdat.tsv')
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
    LT = {'DTYPE_NONE':'None','DTYPE_CATHEDRAL':'Cathedral','DTYPE_CATACOMBS':'Catacombs','DTYPE_CAVES':'Caves','DTYPE_HELL':'Hell','DTYPE_NEST':'Nest','DTYPE_CRYPT':'Crypt'}
    TH = {'':'NONE','THEME_NONE':'NONE','THEME_SHRINE':'Shrine','THEME_GOATSHRINE':'GoatShrine','THEME_BRNCROSS':'BrnCross',
     'THEME_DECAPITATED':'Decapitated','THEME_PURIFYINGFOUNTAIN':'PurifyingFountain','THEME_TEARFOUNTAIN':'TearFountain',
     'THEME_BLOODFOUNTAIN':'BloodFountain','THEME_WEAPONRACK':'WeaponRack','THEME_LIBRARY':'Library','THEME_ARMORSTAND':'ArmorStand',
     'THEME_SKELROOM':'SkelRoom','THEME_MURKYFOUNTAIN':'MurkyFountain','THEME_TORTURE':'Torture'}
    Q = {'':'INVALID','Q_ROCK':'Rock','Q_MUSHROOM':'Mushroom','Q_BUTCHER':'Butcher','Q_BLIND':'Blind','Q_BLOOD':'Blood',
     'Q_ANVIL':'Anvil','Q_WARLORD':'Warlord','Q_PWATER':'PWater','Q_SCHAMB':'Schamb','Q_BETRAYER':'Betrayer'}
    def rust_flag(name):
        return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).upper()
    diffs = 0
    file_idx = {}
    for b, t in zip(blocks, tsv):
        flags = set(re.findall(r'ObjectDataFlags::(\w+)', b))
        if flags == {'NONE'}:
            flags = set()
        got = (int(re.search(r'ofindex: (\d+)', b).group(1)), int(re.search(r'minlvl: (-?\d+)', b).group(1)),
               int(re.search(r'maxlvl: (-?\d+)', b).group(1)),
               re.search(r'olvltype: ([\w:]+)', b).group(1).split('::')[-1],
               re.search(r'otheme: ([\w:\d-]+)', b).group(1).split('::')[-1],
               re.search(r'oquest: ([\w:\d-]+)', b).group(1).split('::')[-1],
               flags, int(re.search(r'anim_delay: (\d+)', b).group(1)),
               int(re.search(r'anim_len: (\d+)', b).group(1)), int(re.search(r'anim_width: (\d+)', b).group(1)))
        exp_of = file_idx.setdefault(t['file'], len(file_idx))
        exp_flags = {rust_flag(f.strip()) for f in t['flags'].split(',') if f.strip()}
        exp = (exp_of, int(t['minLevel']), int(t['maxLevel']), LT.get(t['levelType'] or 'DTYPE_NONE'),
               TH.get(t['theme'] or ''), Q.get(t['quest'] or ''), exp_flags,
               int(t['animDelay'] or 0), int(t['animLen'] or 0), int(t['animWidth'] or 0))
        if got != exp:
            diffs += 1
    print('objdat: %d rows, %d diffs' % (len(blocks), diffs))
    failures += diffs

def check_affixes():
    global failures
    rs = rust_source('item_affix.rs')
    def blocks(start):
        out = []
        i = rs.find(start)
        while i >= 0:
            j = rs.find('AffixData {', i)
            if j < 0:
                break
            depth = 0; k = j + len('AffixData {') - 1
            while k < len(rs):
                if rs[k] == '{':
                    depth += 1
                elif rs[k] == '}':
                    depth -= 1
                    if depth == 0:
                        break
                k += 1
            out.append(rs[j:k+1]); i = k
        return out
    diffs = 0
    for start, tname in [('pub static ITEM_PREFIXES', 'item_prefixes.tsv'), ('pub static ITEM_SUFFIXES', 'item_suffixes.tsv')]:
        tsv = tsv_rows(tname)
        for b, t in zip(blocks(start), tsv):
            got = (re.search(r'name: "([^"]*)"', b).group(1),
                   int(re.search(r'min_level: (\d+)', b).group(1)),
                   int(re.search(r'chance: (\d+)', b).group(1)))
            exp = (t['name'], int(t['minLevel']), int(t['chance']))
            if got != exp:
                diffs += 1
    print('affixes: 178 rows, %d diffs' % diffs)
    failures += diffs

def check_experience():
    global failures
    rs = rust_source('player_dat.rs')
    m = re.search(r'pub const EXP_LEVELS: \[u32; 50\] = \[(.*?)\];', rs, re.S)
    rust_vals = [int(x) for x in re.findall(r'(\d+),', m.group(1))]
    tsv = [int(r[1]) for r in [ln.split('\t') for ln in io.open(os.path.join(TSV, 'Experience.tsv'), encoding='utf-8').read().splitlines()[1:]] if len(r) >= 2]
    diffs = sum(1 for a, b in zip(rust_vals, tsv) if a != b)
    print('Experience: %d rows, %d diffs' % (len(tsv), diffs))
    failures += diffs

def check_quests():
    global failures
    rs = rust_source('quest_new.rs')
    si = rs.find('pub fn get_quest_data'); vi = rs.find('vec![', si)
    # vec![ ... ] block bounds
    depth = 0
    k = vi + 4
    while k < len(rs):
        if rs[k] == '[':
            depth += 1
        elif rs[k] == ']':
            depth -= 1
            if depth == 0:
                break
        k += 1
    limit = k
    blocks = []
    i = rs.find('QuestData {', vi)
    while i >= 0 and i < limit:
        depth = 0; k = i + len('QuestData {') - 1
        while k < len(rs):
            if rs[k] == '{':
                depth += 1
            elif rs[k] == '}':
                depth -= 1
                if depth == 0:
                    break
            k += 1
        blocks.append(rs[i:k+1]); i = rs.find('QuestData {', k)
        if i > si + 60000:
            break
    def num(b, f):
        m = re.search(f + r': (-?\d+)', b)
        return int(m.group(1)) if m else None
    tsv = tsv_rows('questdat.tsv')
    diffs = 0
    for b, t in zip(blocks, tsv):
        got = (num(b, '_qdlvl'), num(b, '_qdmultlvl'),
               re.search(r'_qlvlt: DungeonType::(\w+)', b).group(1),
               num(b, 'quest_book_order'), num(b, '_qdrnd'),
               re.search(r'_qslvl: SetLevel::(\w+)', b).group(1),
               re.search(r'is_single_player_only: (\w+)', b).group(1),
               re.search(r'_qlstr: "([^"]*)"', b).group(1))
        def pasc(s):
            return ''.join(w.capitalize() for w in s.split('_'))
        exp = (int(t['qdlvl']), int(t['qdmultlvl']),
               'None' if not t['qlvlt'] else pasc(t['qlvlt'].replace('DTYPE_', '').lower()),
               int(t['bookOrder']), int(t['qdrnd']),
               'None' if t['qslvl'] == 'SL_NONE'
               else {'SL_SKELKING': 'SkeletonKing', 'SL_BONECHAMB': 'BoneChamber',
                     'SL_MAZE': 'Maze', 'SL_POISONWATER': 'PoisonWater',
                     'SL_VILEBETRAYER': 'VileBetrayer', 'SL_ARENA_CHURCH': 'ArenaChurch',
                     'SL_ARENA_HELL': 'ArenaHell',
                     'SL_ARENA_CIRCLE_OF_LIFE': 'ArenaCircleOfLife'}.get(t['qslvl'], t['qslvl']),
               t['isSinglePlayerOnly'], t['qlstr'])
        if got != exp:
            diffs += 1
    print('quests: %d rows, %d diffs' % (len(blocks), diffs))
    failures += diffs

check_monstdat()
check_spelldat()
check_objdat()
check_affixes()
check_experience()
check_quests()
print('TOTAL DIFFS:', failures)
sys.exit(1 if failures else 0)