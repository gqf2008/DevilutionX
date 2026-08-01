# table_data - authoritative game tables (from upstream/master)

These TSVs are extracted verbatim from diasurgical/DevilutionX upstream/master
(assets/txtdata/...) at the alignment baseline 4b2e6c749 (2026-07). The C++
engine loads them from the game MPQ at runtime; the Rust port embeds the same
data in src/game/*.rs.

- itemdat.tsv - base item data; row order == C++ AllItemsList index (== _item_indexes value).
  gen_items.py regenerates ITEMS_DATA in src/game/item_dat.rs from this file.
- unique_itemdat.tsv - unique item data; row order == UniqueItemId value.
  gen_unique.py regenerates UNIQUE_ITEMS in src/game/item_dat.rs from this file.
- Experience.tsv - per-level XP thresholds; EXP_LEVELS in src/game/player_dat.rs matches it exactly.
- monstdat.tsv / monstdat_hf.tsv - base (112) + Hellfire (26) monster data; gen_monstdat.py
  regenerates MONSTERS_DATA in src/game/monstdat.rs.
- spelldat.tsv / spelldat_hf.tsv - spell data; used for SPELLS_DATA in src/game/spelldat.rs.
- objdat.tsv - object data; ALL_OBJECTS in src/game/objdat.rs.
- item_prefixes.tsv / item_suffixes.tsv - affix data; ITEM_PREFIXES/ITEM_SUFFIXES in src/game/item_affix.rs.
- attrs_*.tsv - per-class attributes; get_class_attributes in src/game/player_dat.rs.
- towners.tsv - towner positions; TOWNER_DATA in src/game/towner.rs.

## Verification

python verify_tables.py compares the Rust tables against these TSVs and exits
non-zero on any diff (monstdat 138, spelldat 52, objdat 109, affixes 178,
Experience 50). This runs in CI (.github/workflows/rust-alignment.yml, data-tables job).
