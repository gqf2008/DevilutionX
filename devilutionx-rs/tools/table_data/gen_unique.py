# -*- coding: utf-8 -*-
import csv, re, sys, io

SRC = r'E:\Users\gxh\Documents\GitHub\DevilutionX'
TSV = SRC + r'\devilutionx-rs\tools\table_data\unique_itemdat.tsv'
HDR = SRC + r'\Source\itemdat.h'

hdr = io.open(HDR, encoding='utf-8', errors='replace').read()
icurs = {}
m = re.search(r'enum item_cursor_graphic : uint8_t \{(.*?)\};', hdr, re.S) or re.search(r'enum item_cursor_graphic \{(.*?)\};', hdr, re.S)
for line in m.group(1).splitlines():
    mm = re.match(r'\s*ICURS_(\w+)\s*=\s*(\d+)', line)
    if mm:
        icurs[mm.group(1)] = int(mm.group(2))

UBI = {'AMULET':'Amulet','ARMOFVAL':'ArmorOfValor','BASTARDSWR':'BastardSword','BATTLEAXE':'BattleAxe',
 'BATTLEBOW':'BattleBow','BOVINE':'Bovine','BREASTPLATE':'BreastPlate','BROADAXE':'BroadAxe',
 'BROADSWR':'BroadSword','BUCKLER':'Buckler','CAPE':'Cape','CHAINMAIL':'ChainMail','CLAYMORE':'Claymore',
 'CLEAVER':'Cleaver','CLOAK':'Cloak','COMPBOW':'CompositeBow','COMPSTAFF':'CompositeStaff','CROWN':'Crown',
 'DAGGER':'Dagger','ELIXIR':'Elixir','FALCHION':'Falchion','FLAIL':'Flail','FULLPLATE':'FullPlate',
 'GOTHSHIELD':'GothicShield','GREATAXE':'GreatAxe','GREATHELM':'GreatHelm','GREATSWR':'GreatSword',
 'GRISWOLD':'Griswold','HARCREST':'HarlequinCrest','HELM':'Helm','HUNTBOW':'HunterBow','INFRARING':'InfraRing',
 'KITESHIELD':'KiteShield','LARGEAXE':'LargeAxe','LARGESHIELD':'LargeShield','LAZSTAFF':'LazarusStaff',
 'LEATHARMOR':'LeatherArmor','LGTFORGE':'LightningForge','LONGBOW':'LongBow','LONGSTAFF':'LongStaff',
 'LONGSWR':'LongSword','MACE':'Mace','MAPOFDOOM':'MapOfDoom','MAUL':'Maul','MORNSTAR':'MorningStar',
 'NONE':'None','OPTAMULET':'OpticAmulet','PLATEMAIL':'PlateMail','QUARSTAFF':'QuarterStaff','RAGS':'Rags',
 'RING':'Ring','ROBE':'Robe','SABRE':'Sabre','SCIMITAR':'Scimitar','SHORTBOW':'ShortBow','SHORTSTAFF':'ShortStaff',
 'SKCROWN':'SkeletonCrown','SKULLCAP':'SkullCap','SMALLAXE':'SmallAxe','SMALLSHIELD':'SmallShield',
 'SPIKCLUB':'SpikedClub','STEELVEIL':'SteelVeil','STUDARMOR':'StuddedArmor','TRING':'TRing',
 'TWOHANDSWR':'TwoHandSword','WARBOW':'WarBow','WARHAMMER':'WarHammer','WARSTAFF':'WarStaff'}

EFFECT = {'TOHIT':'ToHit','TOHIT_CURSE':'ToHitCurse','DAMP':'Damage','DAMP_CURSE':'DamageCurse',
 'TOHIT_DAMP':'ToHitDamage','TOHIT_DAMP_CURSE':'ToHitDamageCurse','ACP':'ArmorPercent',
 'AC_CURSE':'ACCurse','FIRERES':'FireRes','LIGHTRES':'LightRes','MAGICRES':'MagicRes',
 'ALLRES':'AllRes','SPLLVLADD':'SpellLevelAdd','CHARGES':'Charges','FIREDAM':'FireDam',
 'LIGHTDAM':'LightDam','STR':'Str','STR_CURSE':'StrCurse','MAG':'Mag','MAG_CURSE':'MagCurse',
 'DEX':'Dex','DEX_CURSE':'DexCurse','VIT':'Vit','VIT_CURSE':'VitCurse','ATTRIBS':'Attribs',
 'ATTRIBS_CURSE':'AttribsCurse','GETHIT_CURSE':'GetHitCurse','GETHIT':'GetHit','LIFE':'Life',
 'LIFE_CURSE':'LifeCurse','MANA':'Mana','MANA_CURSE':'ManaCurse','DUR':'Durability',
 'DUR_CURSE':'DurabilityCurse','INDESTRUCTIBLE':'Indestructible','LIGHT':'Light',
 'LIGHT_CURSE':'LightCurse','MULT_ARROWS':'MultipleArrows','FIRE_ARROWS':'FireArrows',
 'LIGHT_ARROWS':'LightArrows','THORNS':'Thorns','NOMANA':'NoMana','FIREBALL':'Fireball',
 'ABSHALFTRAP':'AbsHalfTrap','KNOCKBACK':'Knockback','STEALMANA':'StealMana','STEALLIFE':'StealLife',
 'TARGAC':'TargetAC','FASTATTACK':'FastAttack','FASTRECOVER':'FastRecover','FASTBLOCK':'FastBlock',
 'DAMMOD':'DamMod','RNDARROWVEL':'RndArrowVel','SETDAM':'SetDam','SETDUR':'SetDur',
 'NOMINSTR':'NoMinStr','SPELL':'Spell','ONEHAND':'OneHand','3XDAMVDEM':'TripleDemonDamage',
 'ALLRESZERO':'AllResZero','DRAINLIFE':'DrainLife','RNDSTEALLIFE':'RndStealLife','SETAC':'SetAC',
 'ADDACLIFE':'AddACLife','ADDMANAAC':'AddManaAC'}

rows = list(csv.DictReader(io.open(TSV, encoding='utf-8', newline=''), delimiter='\t'))
out = []
for i, r in enumerate(rows):
    cursor = r['cursorGraphic']
    cnum = icurs.get(cursor, 255) if cursor else 255
    powers = []
    npow = 0
    for k in range(6):
        p = r.get('power%d' % k, '')
        if not p:
            powers.append(('Invalid', '0', '0'))
            continue
        npow += 1
        powers.append((EFFECT[p], r.get('power%d.value1' % k, '') or '0', r.get('power%d.value2' % k, '') or '0'))
    name = r['name'].replace('\\', '\\\\').replace('"', '\\"')
    lines = []
    lines.append('    // {:3d} {}'.format(i, r['name']))
    lines.append('    udat!("{}", {}, UniqueBaseItem::{}, {}, {}, {},'.format(
        name, cnum, UBI[r['uniqueBaseItem']], r['minLevel'], npow, r['value']))
    for idx, (t, p1, p2) in enumerate(powers):
        suffix = ',' if idx < 5 else '),'
        lines.append('          ItemEffectType::{}, {}, {}{}'.format(t, p1, p2, suffix))
    out.append('\n'.join(lines))

result = '\n'.join(out)
io.open(SRC + r'\devilutionx-rs\tools\table_data\generated_unique.rs.txt', 'w', encoding='utf-8', newline='\n').write(result)
print('generated', len(rows), 'unique rows')
print(result[:500])