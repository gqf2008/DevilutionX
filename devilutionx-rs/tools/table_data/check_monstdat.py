# -*- coding: utf-8 -*-
import re, io, csv
SRC = r'E:\Users\gxh\Documents\GitHub\DevilutionX'
rs = io.open(SRC + r'\devilutionx-rs\src\game\monstdat.rs', encoding='utf-8').read()
tsv = list(csv.DictReader(io.open(SRC + r'\devilutionx-rs\tools\table_data\monstdat_hf.tsv', encoding='utf-8', newline=''), delimiter='\t'))
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
    if not s.strip(): return 0
    m = 0
    parts = re.split(r'[|,]', s)
    for part in parts:
        part = part.strip()
        if not part:
            continue
        names = re.findall(r'([A-Z_]+)\.0', part)
        if not names:
            names = [part.rsplit('::', 1)[-1]]
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
for i, (entry, row) in enumerate(zip(entries, tsv)):
    args = [a.strip() for a in entry.split(',')]
    def iv(a):
        return int(a) if a.lstrip('-').isdigit() else None
    got = (args[0].strip('"'), iv(args[1]), iv(args[2]), iv(args[3]), iv(args[4]), iv(args[5]),
           AI[args[6].split('::')[1]], iv(args[7]), iv(args[8]), iv(args[9]), iv(args[10]),
           iv(args[11]), iv(args[12]), iv(args[13]), iv(args[14]), CLS[args[15].split('::')[1]],
           res_mask(args[16]), res_mask(args[17]), iv(args[18]))
    r = row
    expected = (r['name'], int(r['minDunLvl']), int(r['maxDunLvl']), int(r['level']),
                int(r['hitPointsMinimum']), int(r['hitPointsMaximum']),
                AI[r['ai']], int(r['intelligence']), int(r['toHit']),
                int(r['minDamage']), int(r['maxDamage']), int(r['toHitSpecial']),
                int(r['minDamageSpecial']), int(r['maxDamageSpecial']), int(r['armorClass']),
                CLS[r['monsterClass']], res_mask(r['resistance']), res_mask(r['resistanceHell']), int(r['exp']))
    if got != expected:
        diffs += 1
        if diffs <= 15:
            print('DIFF row', i, row['_monster_id'])
            for a, b, label in zip(got, expected, ['name','dlvl_min','dlvl_max','lvl','hp_min','hp_max','ai','int','hit','dmg_min','dmg_max','hit_sp','dmg_sp_min','dmg_sp_max','ac','class','res','res_hell','exp']):
                if a != b:
                    print('  ', label, 'rust=', a, 'tsv=', b)
print('total diffs:', diffs)