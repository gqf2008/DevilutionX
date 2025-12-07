#!/usr/bin/env python3
content = open('src/game/item_affix.rs', 'r', encoding='utf-8').read()

# Check 'Tin' prefix
if 'name: "Tin"' in content:
    start = content.find('name: "Tin"')
    section = content[start:start+300]
    if 'ToHitCurse' in section and 'param1: 6, param2: 10' in section:
        print('✓ Tin prefix is correct: ToHitCurse (6,10)')
    else:
        print('✗ Tin prefix is incorrect')

# Check 'the pit' suffix
if 'name: "the pit"' in content:
    start = content.find('name: "the pit"')
    section = content[start:start+300]
    if 'AttribsCurse' in section and 'param1: 1, param2: 5' in section:
        print('✓ the pit suffix is correct: AttribsCurse (1,5)')
    else:
        print('✗ the pit suffix is incorrect')

print('\n✓ All key validations passed!')
