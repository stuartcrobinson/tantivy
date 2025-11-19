# 's' Type Byte Corruption - Implementation Decision

**Date:** 2025-11-19
**Status:** ACCEPTED - Ship with corruption
**Risk Level:** Low (cosmetic only)

## The Bug

Edge ngram tokens indexed with 's' type byte corruption:
- **Expected:** `"title\0la"`, `"title\0lap"`, `"title\0lapt"`
- **Actual:** `"title\0sla"`, `"title\0slap"`, `"title\0slapt"`

The 's' type byte (0x73) appears between JSON path and token text.

## Root Cause

`src/core/json_utils.rs` line ~138:
```rust
set_path_id(term_buffer, unordered_id);  // buffer = [field][j][unordered_id]
set_type(term_buffer, Type::Str);        // buffer = [field][j][unordered_id][s]
let path_end = term_buffer.len_bytes();  // = 5 (includes 's')
postings_writer.index_text(..., path_end);

// Inside index_text():
term_buffer.truncate_value_bytes(path_end);  // Preserves [field][j][unordered_id][s]
term_buffer.append_bytes(token.text);         // Appends after 's'
```

Result: `[unordered_id][s][token]` → serialized as `"path\0stoken"`

## Why It Works Anyway

**Symmetric corruption:** Query construction has same bug.
- Indexed term: `"title\0slap"` (has 's')
- Query term: `"\0\0\0\0jtitle\0slap"` (also has 's' in same relative position)
- Term lookup succeeds because both sides match

**Evidence:**
- `tests/compare_term_bytes.rs` - Direct term lookup returns FOUND
- `tests/edge_ngram_e2e_spike.rs` - Queries return correct results (1 hit)
- All 3 working tests confirm queries match corrupted terms

## Decision: Accept and Ship

**Why not fix:**
1. **Zero functional impact** - queries work correctly
2. **Symmetric bug** - both indexing and querying corrupt identically
3. **High fix cost** - term structure is complex, 20+ hours estimated
4. **No user visibility** - term bytes internal only
5. **Performance negligible** - one extra byte per term

**Why this is safe:**
- Flapjack controls both indexing and querying
- No index sharing with upstream Tantivy
- Users never see term bytes
- If upstream fixes, we can merge

**What we're NOT doing:**
- NOT fixing term.rs structure
- NOT modifying postings_writer.rs truncation logic
- NOT changing json_utils.rs set_type timing

## Upstream Status

Bug exists in upstream Tantivy commit `70e591e23` (Oct 2024). Not fork-introduced. Upstream JSON text search likely untested with multi-token inputs.

## Test Updates

Modified `tests/edge_ngram_e2e_spike.rs`:
- Removed `test_edge_ngram_indexing_no_corruption` assertion against 's'
- Tests now validate queries work, ignore byte-level corruption

## Monitoring

If future Tantivy upgrade fixes corruption:
1. Our queries will break (term mismatch)
2. We'll need to reindex OR patch query construction to remove 's'
3. Mitigation: Test suite will catch this immediately

## Documentation

Preserved for future reference:
- `stuart/CRITICAL_FINDINGS.md` - Original bug discovery
- `stuart/TEST_FINDINGS_2025_11_19.md` - Empirical validation
- `stuart/JSON_TEXT_TOKENIZATION_COMPREHENSIVE_ANALYSIS.md` - Deep dive (historical)

## Sign-off

**Approved by:** Stuart (2025-11-19)
**Reasoning:** Pragmatic - focus on Algolia parity, not perfect term encoding
**Impact:** None on Flapjack functionality
**Risk:** Low - symmetric bug, validated working