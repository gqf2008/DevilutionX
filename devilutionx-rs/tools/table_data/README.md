# table_data - authoritative game tables (from upstream/master)

These TSVs are extracted verbatim from diasurgical/DevilutionX upstream/master
(assets/txtdata/...) at the alignment baseline 4b2e6c749 (2026-07). The C++
engine loads them from the game MPQ at runtime; the Rust port embeds the same
data in src/game/*.rs.

- itemdat.tsv - base item data; row order == C++ AllItemsList index (== _item_indexes value).
  gen_items.py regenerates ITEMS_DATA in src/game/item_dat.rs from this file.
- Experience.tsv - per-level XP thresholds; EXP_LEVELS in src/game/player_dat.rs matches it exactly.
- unique_itemdat.tsv - unique item data (used for the UNIQUE_ITEMS port).

Regenerate: python tools/table_data/gen_items.py (then splice the emitted
generated_items.rs.txt into src/game/item_dat.rs).
