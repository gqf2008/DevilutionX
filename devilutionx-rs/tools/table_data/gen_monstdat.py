# -*- coding: utf-8 -*-
import csv, io

SRC = r'E:\Users\gxh\Documents\GitHub\DevilutionX'
TSV = SRC + r'\devilutionx-rs\tools\table_data\monstdat_hf.tsv'

RES_ORDER = ['RESIST_MAGIC','RESIST_FIRE','RESIST_LIGHTNING','IMMUNE_MAGIC','IMMUNE_FIRE','IMMUNE_LIGHTNING','IMMUNE_ACID']
def res_expr(s):
    parts = [p.strip() for p in s.split(',') if p.strip()]
    if not parts:
        return 'MonsterResistance::NONE'
    if len(parts) == 1:
        return 'MonsterResistance::' + parts[0]
    inner = ' | '.join('MonsterResistance::%s.0' % p for p in parts)
    return 'MonsterResistance(%s)' % inner

rows = list(csv.DictReader(io.open(TSV, encoding='utf-8', newline=''), delimiter='\t'))
out = []
for i, r in enumerate(rows):
    mid = r['_monster_id']
    line = ('    // %d: %s\n' % (i, mid) +
            '    mdat!("%s", %s, %s, %s, %s, %s, MonsterAIID::%s, %s, %s, %s, %s, %s, %s, %s, %s,\n' % (
                r['name'], r['minDunLvl'], r['maxDunLvl'], r['level'],
                r['hitPointsMinimum'], r['hitPointsMaximum'], r['ai'],
                r['intelligence'], r['toHit'], r['minDamage'], r['maxDamage'],
                r['toHitSpecial'], r['minDamageSpecial'], r['maxDamageSpecial'], r['armorClass']) +
            '          MonsterClass::%s, %s, %s, %s, %s,
          %s, %s),' % (
                r['monsterClass'], res_expr(r['resistance']), res_expr(r['resistanceHell']), r['exp'], r['image'],
                '[' + r['frames[6]'] + ']', '[' + r['rate[6]'] + ']'))
    out.append(line)

result = '\n'.join(out)
io.open(SRC + r'\devilutionx-rs\tools\table_data\generated_monstdat.rs.txt', 'w', encoding='utf-8', newline='\n').write(result)
print('generated', len(rows), 'monster rows')
print(result[:600])