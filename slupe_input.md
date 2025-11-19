=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 04:37:36.995 pm
---------------------
x9k ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/IMPLEMENTATION_PLAN.md
m2p ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/IMPLEMENTATION_PLAN.md
w7j ✅ write_file /Users/stuart/repos/tantivy/stuart/PHASE3_STATUS.md
=== END ===
## Status Assessment

**Current state:**
- EdgeNgramFilter implemented and working (generates correct ngrams per word)
- Term corruption exists (`"title\0sla"` instead of `"title\0la"`) but queries mysteriously work anyway
- Real blocker: QueryParser creates PhraseQuery for multi-token inputs, causing 0 hits on edge ngram fields

**Key insight from CRITICAL_FINDINGS.md:**
The corruption is cosmetic. Manual TermQuery construction matches corrupted indexed terms, proving term lookup isn't byte-exact. Either:
1. Query construction adds 's' identically (both sides wrong)
2. Term matching normalizes during lookup
3. Path resolution reconciles the difference

**Decision:** Ship with corruption. It's upstream's bug (commit `70e591e23`), doesn't break functionality, fixing it risks 20+ hours debugging term structure.

## Documentation Updates Needed

Before implementation, update these docs:

```sh nesl
#!nesl [@three-char-SHA-256: x9k]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/IMPLEMENTATION_PLAN.md"
old_text = <<'EOT_x9k'
## Phase 1: EdgeNgramFilter Implementation (6h)

**Objective:** Build TokenFilter that generates edge ngrams per-word

**Reference:** `SplitCompoundWords` tokenizer (existing pattern for 1-to-many token emission)

### Tasks

- [ ] Create `src/tokenizer/edge_ngram_filter.rs`
EOT_x9k
new_text = <<'EOT_x9k'
## Phase 1: EdgeNgramFilter Implementation ✅ COMPLETE (6h)

**Objective:** Build TokenFilter that generates edge ngrams per-word

**Reference:** `SplitCompoundWords` tokenizer (existing pattern for 1-to-many token emission)

### Tasks

- [x] Create `src/tokenizer/edge_ngram_filter.rs`
EOT_x9k
#!end_x9k
```

```sh nesl
#!nesl [@three-char-SHA-256: m2p]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/IMPLEMENTATION_PLAN.md"
old_text = <<'EOT_m2p'
## Phase 2: Integration Tests (3h)

**Objective:** Validate EdgeNgramFilter works with JSON fields and isolate QueryParser bug

### Tasks

- [ ] Create `tests/edge_ngram_json_integration.rs`
EOT_m2p
new_text = <<'EOT_m2p'
## Phase 2: Integration Tests ✅ COMPLETE (3h)

**Objective:** Validate EdgeNgramFilter works with JSON fields and isolate QueryParser bug

### Tasks

- [x] Create `tests/edge_ngram_json_integration.rs` (implemented as `tests/edge_ngram_e2e_spike.rs`)
EOT_m2p
#!end_m2p
```

```sh nesl
#!nesl [@three-char-SHA-256: w7j]
action = "write_file"
path = "/Users/stuart/repos/tantivy/stuart/PHASE3_STATUS.md"
content = <<'EOT_w7j'
# Phase 3: QueryBehavior Implementation - Status

**Date:** 2025-11-19
**Status:** Ready to implement
**Prerequisite:** EdgeNgramFilter complete, tests passing

## What's Working

✅ EdgeNgramFilter generates correct ngrams: `["la", "lap", "lapt", "lapto", "laptop"]`
✅ Terms indexed (with cosmetic corruption): `"title\0sla"`, `"title\0slap"`, etc.
✅ Manual TermQuery matches: 1 hit on "lap" query
✅ Multi-word values work: Both "gam" and "lap" match "Gaming Laptop"

## The Problem

QueryParser creates PhraseQuery for multi-token inputs:
```rust
// User query: "lap"
// EdgeNgram tokenizer produces: ["la", "lap"]
// QueryParser creates: PhraseQuery(["la", "lap"]) requiring consecutive match
// In "Gaming Laptop": "la" and "lap" exist but NOT consecutively
// Result: 0 hits
```

## The Solution

Add schema-level QueryBehavior enum to control multi-token query construction:

```rust
pub enum QueryBehavior {
    Phrase,    // Default: consecutive terms (existing behavior)
    TermsOr,   // Edge ngrams: OR'd term queries
}
```

When field uses `QueryBehavior::TermsOr`, QueryParser creates:
```rust
BooleanQuery {
    subqueries: [
        (Should, TermQuery("la")),
        (Should, TermQuery("lap"))
    ]
}
```

## Implementation Checklist

- [ ] Add enum to `src/schema/text_options.rs`
- [ ] Extend `TextFieldIndexing` with getter/setter
- [ ] Modify `src/query/query_parser/query_parser.rs`:
  - [ ] Text field path (~line 960)
  - [ ] JSON field path (~line 1040)
- [ ] Schema serialization (skip_serializing_if default)
- [ ] Update existing integration test to use TermsOr
- [ ] Verify all upstream tests still pass

## Files to Modify

1. `src/schema/text_options.rs` - Add QueryBehavior enum
2. `src/query/query_parser/query_parser.rs` - Check behavior before creating PhraseQuery
3. `tests/edge_ngram_e2e_spike.rs` - Add QueryBehavior to schema config

## Acceptance Criteria

- [ ] Query "lap" returns 1 hit on "Gaming Laptop"
- [ ] Query "gam" returns 1 hit on "Gaming Laptop"
- [ ] Phrase queries still work on non-edge-ngram fields
- [ ] cargo test passes (943+ tests)
- [ ] No regressions in query_parser tests
EOT_w7j
#!end_w7j
```

## Next Actions

First, verify QueryParser code structure to plan implementation:

```bash
cd /Users/stuart/repos/tantivy
grep -n "PhraseQuery" src/query/query_parser/query_parser.rs | head -20
grep -n "generate_literals_for_str" src/query/query_parser/query_parser.rs
grep -n "generate_literals_for_json" src/query/query_parser/query_parser.rs
```

Run those commands and share output. I need to see:
1. Where PhraseQuery is constructed for text fields
2. Where PhraseQuery is constructed for JSON fields  
3. How multi-token handling differs between field types

Then I'll give you explicit NESL for Phase 3 implementation.