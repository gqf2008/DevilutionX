#!/usr/bin/env python3
"""Generate Rust affix arrays from TSV files"""

import csv
from typing import Dict, List, Tuple

def parse_item_types(item_types_str: str) -> str:
    """Convert comma-separated item types to Rust bitflags"""
    if not item_types_str:
        return "AffixItemType::NONE.0"

    type_mapping = {
        "Weapon": "AffixItemType::ALL_WEAPONS.0",
        "Bow": "AffixItemType::BOW.0",
        "Staff": "AffixItemType::STAFF.0",
        "Armor": "AffixItemType::ALL_ARMOR.0",
        "Shield": "AffixItemType::SHIELD.0",
        "Misc": "AffixItemType::JEWELRY.0",
    }

    types = [t.strip() for t in item_types_str.split(',')]
    rust_types = []

    for t in types:
        if t in type_mapping:
            rust_types.append(type_mapping[t])

    if not rust_types:
        return "AffixItemType::NONE.0"

    if len(rust_types) == 1:
        return rust_types[0]

    return " | ".join(rust_types)

def parse_effect_type(power: str) -> str:
    """Convert TSV power string to Rust ItemEffectType variant"""
    mapping = {
        "TOHIT": "ToHit",
        "TOHIT_CURSE": "ToHitCurse",
        "DAMP": "Damage",
        "DAMP_CURSE": "DamageCurse",
        "TOHIT_DAMP": "ToHitDamage",
        "TOHIT_DAMP_CURSE": "ToHitDamageCurse",
        "ACP": "ArmorPercent",
        "ACP_CURSE": "ArmorPercentCurse",
        "FIRERES": "FireRes",
        "LIGHTRES": "LightRes",
        "MAGICRES": "MagicRes",
        "ALLRES": "AllRes",
        "MANA": "Mana",
        "MANA_CURSE": "ManaCurse",
        "SPLLVLADD": "SpellLevelAdd",
        "CHARGES": "Charges",
        "FIREDAM": "FireDam",
        "LIGHTDAM": "LightDam",
        "DAMMOD": "DamMod",
        "GETHIT": "GetHit",
        "GETHIT_CURSE": "GetHitCurse",
        "STR": "Str",
        "STR_CURSE": "StrCurse",
        "DEX": "Dex",
        "DEX_CURSE": "DexCurse",
        "MAG": "Mag",
        "MAG_CURSE": "MagCurse",
        "VIT": "Vit",
        "VIT_CURSE": "VitCurse",
        "ATTRIBS": "Attribs",
        "ATTRIBS_CURSE": "AttribsCurse",
        "LIFE": "Life",
        "LIFE_CURSE": "LifeCurse",
        "DUR": "Durability",
        "DUR_CURSE": "DurabilityCurse",
        "INDESTRUCTIBLE": "Indestructible",
        "LIGHT": "Light",
        "LIGHT_CURSE": "LightCurse",
        "FIRE_ARROWS": "FireArrows",
        "LIGHT_ARROWS": "LightArrows",
        "THORNS": "Thorns",
        "NOMANA": "NoMana",
        "ABSHALFTRAP": "AbsHalfTrap",
        "KNOCKBACK": "Knockback",
        "STEALMANA": "StealMana",
        "STEALLIFE": "StealLife",
        "TARGAC": "TargetAC",
        "FASTATTACK": "FastAttack",
        "FASTRECOVER": "FastRecover",
        "FASTBLOCK": "FastBlock",
    }

    return mapping.get(power, power)

def parse_alignment(alignment: str) -> str:
    """Convert alignment string to Rust GoodOrEvil"""
    if alignment == "Good":
        return "GoodOrEvil::Good"
    elif alignment == "Evil":
        return "GoodOrEvil::Evil"
    else:
        return "GoodOrEvil::Any"

def generate_affix_data(row: Dict[str, str], seen_names: Dict[str, int]) -> str:
    """Generate a single AffixData struct from TSV row"""
    name = row['name']
    power = row['power']
    param1 = row['power.value1'] or '0'
    param2 = row['power.value2'] or '0'
    min_level = row['minLevel'] or '0'
    item_types = parse_item_types(row['itemTypes'])
    alignment = parse_alignment(row['alignment'])
    chance = row['chance'] or '1'
    is_good = 'true' if row['useful'] == 'true' else 'false'
    min_val = row['minVal'] or '0'
    max_val = row['maxVal'] or '0'
    mult_val = row['multVal'] or '1'

    effect_type = parse_effect_type(power)

    # Handle duplicate names with comments
    comment = ""
    if name in seen_names:
        seen_names[name] += 1
        comment = f" // {name} #{seen_names[name]}"
    else:
        seen_names[name] = 1

    return f'''    AffixData {{{comment}
        name: "{name}",
        power: ItemPower {{ effect_type: ItemEffectType::{effect_type}, param1: {param1}, param2: {param2} }},
        min_level: {min_level},
        item_types: AffixItemType({item_types}),
        is_good: {is_good},
        alignment: {alignment},
        min_val: {min_val}, max_val: {max_val}, mult_val: {mult_val},
        chance: {chance},
    }},'''

def generate_affixes_from_tsv(tsv_path: str, array_name: str) -> Tuple[str, int]:
    """Generate Rust array content from TSV file"""
    affixes = []
    seen_names: Dict[str, int] = {}

    with open(tsv_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f, delimiter='\t')
        for row in reader:
            affix_code = generate_affix_data(row, seen_names)
            affixes.append(affix_code)

    return '\n'.join(affixes), len(affixes)

def main():
    # Generate prefixes
    prefix_path = r"d:\Users\gxh\Documents\GitHub\DevilutionX\assets\txtdata\items\item_prefixes.tsv"
    suffix_path = r"d:\Users\gxh\Documents\GitHub\DevilutionX\assets\txtdata\items\item_suffixes.tsv"

    print("Generating ITEM_PREFIXES...")
    prefixes_code, prefix_count = generate_affixes_from_tsv(prefix_path, "ITEM_PREFIXES")

    print("Generating ITEM_SUFFIXES...")
    suffixes_code, suffix_count = generate_affixes_from_tsv(suffix_path, "ITEM_SUFFIXES")

    # Write to output files
    with open('prefixes.txt', 'w', encoding='utf-8') as f:
        f.write(prefixes_code)

    with open('suffixes.txt', 'w', encoding='utf-8') as f:
        f.write(suffixes_code)

    print(f"\nGeneration complete!")
    print(f"Prefixes: {prefix_count}/83")
    print(f"Suffixes: {suffix_count}/95")
    print(f"\nOutput written to prefixes.txt and suffixes.txt")

if __name__ == '__main__':
    main()
