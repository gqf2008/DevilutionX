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
- monstdat.tsv / unique_monstdat.tsv - monster base/unique data (extracted for the
  upcoming monstdat alignment pass; not yet ported).

Regenerate: python tools/table_data/gen_items.py / gen_unique.py (then splice the
emitted generated_*.rs.txt into src/game/item_dat.rs).
