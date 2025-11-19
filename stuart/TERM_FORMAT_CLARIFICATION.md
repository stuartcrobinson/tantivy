# JSON Text Field Term Format - Clarification

**Date:** 2025-11-19
**Status:** Understanding corrected
**Previous belief:** 's' byte was corruption
**Reality:** 's' byte is correct type indicator

## Actual Term Structure

For JSON text fields, terms are encoded as:
```
[field:4][type='j':1][path_string][path_terminator:\x00][value_type:'s':1][token_text]
```

**Example:** Document with `{"title": "Laptop"}`
- Indexed term for token "lap": `"title\0slap"`
- Breakdown: `"title" + \x00 + 's' + "lap"`

## Why We Thought It Was Corruption

**Misleading display:** When printed, `"title\0slap"` looks like path+token with 's' in wrong place.

**Expected (wrongly):** `"title\0lap"` (no 's')

**Actual (correct):** `"title\0slap"` ('s' = Type::Str indicator)

## Source of Truth

**Code evidence:**
```rust
// src/schema/term.rs
pub fn append_type_and_str(&mut self, val: &str) {
    self.0.push(Type::Str.to_code());  // Adds 's' (0x73)
    self.0.extend(val.as_bytes().as_ref());
}

// src/schema/field_type.rs line 150
b's' => Some(Type::Str)
```

**Design docs:** Issue #1251, json.md document this format (though unclearly).

## Query Matching

Queries work because:
1. Indexing: `set_type(Type::Str)` → adds 's' → `"title\0slap"`
2. Query: `append_type_and_str("lap")` → adds 's' → `"title\0slap"`
3. Both construct identical terms → lookup succeeds

**Not a symmetric bug** - just correct implementation on both sides.

## Lessons

1. Always check source before assuming bugs
2. Term byte format is internal - display can be misleading
3. Type indicators are part of Tantivy's multi-type field design

## Files to Ignore

Historical analysis (wrong assumptions):
- `CRITICAL_FINDINGS.md` - claimed 's' corruption
- `TEST_FINDINGS_2025_11_19.md` - "symmetric corruption" theory
- `JSON_TEXT_TOKENIZATION_*.md` - debugging non-existent bug

Keep for historical reference but don't use for implementation decisions.