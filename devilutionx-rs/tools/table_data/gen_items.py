# -*- coding: utf-8 -*-
import csv, re, sys

SRC = r'E:\Users\gxh\Documents\GitHub\DevilutionX'
TSV = SRC + r'\devilutionx-rs\tools\table_data\itemdat.tsv'
HDR = SRC + r'\Source\itemdat.h'

hdr = open(HDR, encoding='utf-8', errors='replace').read()
icurs = {}
m = re.search(r'enum item_cursor_graphic : uint8_t \{(.*?)\};', hdr, re.S) or re.search(r'enum item_cursor_graphic \{(.*?)\};', hdr, re.S)
for line in m.group(1).splitlines():
    mm = re.match(r'\s*ICURS_(\w+)\s*=\s*(\d+)', line)
    if mm:
        icurs[mm.group(1)] = int(mm.group(2))
idi = {}
m = re.search(r'enum _item_indexes : int16_t \{(.*?)\};', hdr, re.S) or re.search(r'enum _item_indexes \{(.*?)\};', hdr, re.S)
cur = 0
for line in m.group(1).splitlines():
    line = line.split('//')[0]
    mm = re.match(r'\s*(\w+)\s*(?:=\s*(\d+))?\s*,', line)
    if mm and mm.group(1).startswith('IDI_'):
        name, val = mm.group(1), mm.group(2)
        if val is not None:
            cur = int(val)
        idi[name] = cur
        cur += 1

CLASS = {'Gold':'Gold','Weapon':'Weapon','Armor':'Armor','Misc':'Misc','Quest':'Quest'}
EQUIP = {'Unequippable':'Unequipable','One-handed':'OneHand','Two-handed':'TwoHand',
         'Armor':'Armor','Helm':'Helm','Ring':'Ring','Amulet':'Amulet'}
ITYPE = {'Gold':'Gold','Sword':'Sword','Axe':'Axe','Bow':'Bow','Mace':'Mace','Shield':'Shield',
         'LightArmor':'LightArmor','Helm':'Helm','MediumArmor':'MediumArmor','HeavyArmor':'HeavyArmor',
         'Staff':'Staff','Ring':'Ring','Amulet':'Amulet','Misc':'Misc'}
UBI = {'AMULET':'Amulet','ARMOFVAL':'ArmorOfValor','BASTARDSWR':'BastardSword','BATTLEAXE':'BattleAxe',
 'BATTLEBOW':'BattleBow','BOVINE':'Bovine','BREASTPLATE':'BreastPlate','BROADAXE':'BroadAxe',
 'BROADSWR':'BroadSword','BUCKLER':'Buckler','CAPE':'Cape','CHAINMAIL':'ChainMail','CLAYMORE':'Claymore',
 'CLEAVER':'Cleaver','CLOAK':'Cloak','COMPBOW':'CompositeBow','COMPSTAFF':'CompositeStaff','CROWN':'Crown',
 'DAGGER':'Dagger','ELIXIR':'Elixir','FALCHION':'Falchion','FLAIL':'Flail','FULLPLATE':'FullPlate',
 'GOTHSHIELD':'GothicShield','GREATAXE':'GreatAxe','GREATHELM':'GreatHelm','GREATSWR':'GreatSword',
 'GRISWOLD':'Griswold','HARCREST':'HarlequinCrest','HELM':'Helm','HUNTBOW':'HunterBow','INFRARING':'InfraRing',
 'KITESHIELD':'KiteShield','LARGEAXE':'LargeAxe','LARGESHIELD':'LargeShield','LAZSTAFF':'LazarusStaff',
 'LEATHARMOR':'LeatherArmor','LONGBOW':'LongBow','LONGSTAFF':'LongStaff','LONGSWR':'LongSword','MACE':'Mace',
 'MAPOFDOOM':'MapOfDoom','MAUL':'Maul','MORNSTAR':'MorningStar','NONE':'None','OPTAMULET':'OpticAmulet',
 'PLATEMAIL':'PlateMail','QUARSTAFF':'QuarterStaff','RAGS':'Rags','RING':'Ring','ROBE':'Robe','SABRE':'Sabre',
 'SCIMITAR':'Scimitar','SHORTBOW':'ShortBow','SHORTSTAFF':'ShortStaff','SKCROWN':'SkeletonCrown',
 'SKULLCAP':'SkullCap','SMALLAXE':'SmallAxe','SMALLSHIELD':'SmallShield','SPIKCLUB':'SpikedClub',
 'STEELVEIL':'SteelVeil','STUDARMOR':'StuddedArmor','TRING':'TRing','TWOHANDSWR':'TwoHandSword',
 'WARBOW':'WarBow','WARHAMMER':'WarHammer','WARSTAFF':'WarStaff'}
MISC = {'AMULET':'Amulet','ARENAPOT':'ArenaPot','AURIC':'AuricAmulet','BOOK':'Book','EAR':'Ear',
 'ELIXDEX':'ElixirDex','ELIXMAG':'ElixirMag','ELIXSTR':'ElixirStr','ELIXVIT':'ElixirVit','FULLHEAL':'FullHeal',
 'FULLMANA':'FullMana','FULLREJUV':'FullRejuv','GR_RUNEF':'GrRuneF','GR_RUNEL':'GrRuneL','HEAL':'Heal',
 'MANA':'Mana','MAPOFDOOM':'MapOfDoom','NONE':'None','NOTE':'Note','OILACC':'OilAcc','OILBSMTH':'OilBSmith',
 'OILOF':'OilOf','OILSHARP':'OilSharp','REJUV':'Rejuv','RING':'Ring','RUNEF':'RuneF','RUNEL':'RuneL',
 'RUNES':'RuneS','SCROLL':'Scroll','SCROLLT':'ScrollT','SPECELIX':'SpecElixir','STAFF':'Staff','UNIQUE':'Unique'}
SPELL = {'Apocalypse':'Apocalypse','ChainLightning':'ChainLightning','ChargedBolt':'ChargedBolt',
 'FireWall':'FireWall','Fireball':'Fireball','FlameWave':'FlameWave','Flash':'Flash','Golem':'Golem',
 'Guardian':'Guardian','Healing':'Healing','Identify':'Identify','Inferno':'Inferno','Infravision':'Infravision',
 'Lightning':'Lightning','Mana':'Mana','ManaShield':'ManaShield','Nova':'Nova','Null':'Null','Phasing':'Phasing',
 'Resurrect':'Resurrect','Search':'Search','StoneCurse':'StoneCurse','Teleport':'Teleport','TownPortal':'TownPortal'}
FX = {'RandomStealLife':'RANDOM_STEAL_LIFE'}

rows = list(csv.DictReader(open(TSV, encoding='utf-8', newline=''), delimiter='\t'))
out = []
for i, r in enumerate(rows):
    idname = next((k for k, v in idi.items() if v == i), '')
    label = idname if idname else (r['name'] or '')
    cursor = r['cursorGraphic']
    cnum = icurs.get(cursor, 255) if cursor else 255
    fx = r['specialEffects'] or ''
    fxs = ' | '.join('ItemSpecialEffect::' + FX[e] for e in fx.split('|') if e) or 'ItemSpecialEffect::NONE'
    def rust_str(s):
        return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'
    ubi = UBI[r['uniqueBaseItem']]
    line = ('    // {:3d} {}'.format(i, label) + '\n' +
            '    idat!({}, ItemClass::{}, ItemEquipType::{}, {}, ItemType::{},'.format(
                r['dropRate'], CLASS[r['class']], EQUIP[r['equipType']], cnum, ITYPE[r['itemType']]) + '\n' +
            '          UniqueBaseItem::{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {},'.format(
                ubi, rust_str(r['name']), rust_str(r['shortName']), r['minMonsterLevel'],
                r['durability'], r['minDamage'], r['maxDamage'], r['minArmor'], r['maxArmor'],
                r['minStrength'], r['minMagic'], r['minDexterity']) + '\n' +
            '          {}, ItemMiscId::{}, SpellID::{}, {}, {}),'.format(
                fxs, MISC[r['miscId']], SPELL[r['spell']], r['usable'], r['value']))
    out.append(line)

result = '\n'.join(out)
open(SRC + r'\devilutionx-rs\tools\table_data\generated_items.rs.txt', 'w', encoding='utf-8', newline='\n').write(result)
print('generated', len(rows), 'rows')