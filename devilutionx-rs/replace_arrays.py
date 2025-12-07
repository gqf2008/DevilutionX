#!/usr/bin/env python3
"""Replace ITEM_PREFIXES and ITEM_SUFFIXES in item_affix.rs"""

# Read the generated prefix and suffix code
with open('prefixes.txt', 'r', encoding='utf-8') as f:
    prefixes_content = f.read().strip()

with open('suffixes.txt', 'r', encoding='utf-8') as f:
    suffixes_content = f.read().strip()

# Read the original file
with open('src/game/item_affix.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Find ITEM_PREFIXES range
prefix_start_idx = 113  # line 114 (0-indexed)
prefix_end_idx = None
for i in range(prefix_start_idx, len(lines)):
    if lines[i].strip() == '];':
        prefix_end_idx = i
        break

print(f'Found ITEM_PREFIXES: lines {prefix_start_idx+1} to {prefix_end_idx+1}')

# Find ITEM_SUFFIXES range (starts after a few comment lines)
suffix_start_idx = None
for i in range(prefix_end_idx + 1, len(lines)):
    if 'pub static ITEM_SUFFIXES' in lines[i]:
        suffix_start_idx = i
        break

suffix_end_idx = None
for i in range(suffix_start_idx, len(lines)):
    if lines[i].strip() == '];':
        suffix_end_idx = i
        break

print(f'Found ITEM_SUFFIXES: lines {suffix_start_idx+1} to {suffix_end_idx+1}')

# Build new file content
new_lines = []

# Part 1: Before ITEM_PREFIXES
new_lines.extend(lines[:prefix_start_idx+1])  # Include "pub static ITEM_PREFIXES: &[AffixData] = &["

# Part 2: New ITEM_PREFIXES content
new_lines.append(prefixes_content + '\n')

# Part 3: Between arrays (close PREFIXES and comments before SUFFIXES)
new_lines.extend(lines[prefix_end_idx:suffix_start_idx+1])  # Include "];" and comments and "pub static ITEM_SUFFIXES: &[AffixData] = &["

# Part 4: New ITEM_SUFFIXES content
new_lines.append(suffixes_content + '\n')

# Part 5: Rest of file
new_lines.extend(lines[suffix_end_idx:])  # Include "];" and everything after

# Write the new file
with open('src/game/item_affix.rs', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)

print(f'\n✓ Replacement complete!')
print(f'  - ITEM_PREFIXES: {prefix_end_idx - prefix_start_idx - 1} old lines → (check new count)')
print(f'  - ITEM_SUFFIXES: {suffix_end_idx - suffix_start_idx - 1} old lines → (check new count)')
print(f'  - Total file lines: {len(lines)} → {len(new_lines)}')
