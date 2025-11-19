=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:38:36.975 pm
---------------------
q7n ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/research/algolia_test_1/critical_test_v2.js
=== END ===
Levenshtein is separate - it's for fuzzy matching (typos), not prefix. Don't test together.

Add multi-word tests to Algolia:

```sh nesl
#!nesl [@three-char-SHA-256: q7n]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/research/algolia_test_1/critical_test_v2.js"
old_text = <<'EOT_q7n'
  console.log('\nTest 5: Does "lap" match document "La" (only has "la" ngram)?');
  const result5 = await client.search({
    requests: [{ indexName, query: 'lap' }]
  });
  const hasLa = result5.results[0].hits.some(h => h.title === 'La');
  console.log(`Query "lap" matches "La": ${hasLa}`);
  if (hasLa) {
    console.log('→ Query IS tokenized to ngrams ["la", "lap"]');
    console.log('→ EdgeNgramFilter needed on BOTH index and query');
  } else {
    console.log('→ Query stays as single term "lap"');
    console.log('→ EdgeNgramFilter ONLY on index, NOT on query');
  }
EOT_q7n
new_text = <<'EOT_q7n'
  console.log('\nTest 5: Does "lap" match document "La" (only has "la" ngram)?');
  const result5 = await client.search({
    requests: [{ indexName, query: 'lap' }]
  });
  const hasLa = result5.results[0].hits.some(h => h.title === 'La');
  console.log(`Query "lap" matches "La": ${hasLa}`);
  if (hasLa) {
    console.log('→ Query IS tokenized to ngrams ["la", "lap"]');
    console.log('→ EdgeNgramFilter needed on BOTH index and query');
  } else {
    console.log('→ Query stays as single term "lap"');
    console.log('→ EdgeNgramFilter ONLY on index, NOT on query');
  }

  console.log('\n=== MULTI-WORD QUERY TESTS ===\n');
  
  console.log('Test 6: "gaming laptop" (full multi-word)');
  const r6 = await client.search({
    requests: [{ indexName, query: 'gaming laptop' }]
  });
  console.log(`Query "gaming laptop": ${r6.results[0].hits.length} hits`);
  console.log('  Matches:', r6.results[0].hits.map(h => h.title).join(', '));
  
  console.log('\nTest 7: "gam lap" (partial prefixes on both words)');
  const r7 = await client.search({
    requests: [{ indexName, query: 'gam lap' }]
  });
  console.log(`Query "gam lap": ${r7.results[0].hits.length} hits`);
  console.log('  Matches:', r7.results[0].hits.map(h => h.title).join(', '));
  
  console.log('\nTest 8: "laptop gaming" (reversed order)');
  const r8 = await client.search({
    requests: [{ indexName, query: 'laptop gaming' }]
  });
  console.log(`Query "laptop gaming": ${r8.results[0].hits.length} hits`);
  console.log('  Matches:', r8.results[0].hits.map(h => h.title).join(', '));
  
  console.log('\nTest 9: "mouse stand" (terms from different docs)');
  const r9 = await client.search({
    requests: [{ indexName, query: 'mouse stand' }]
  });
  console.log(`Query "mouse stand": ${r9.results[0].hits.length} hits`);
  console.log('  Matches:', r9.results[0].hits.map(h => h.title).join(', '));
  
  console.log('\n=== INTERPRETATION ===');
  console.log('If multi-word queries match individual docs: AND semantics (each term must match)');
  console.log('If match multiple docs: OR semantics or ranking-based');
  console.log('Order matters: phrase-like vs bag-of-words\n');
EOT_q7n
#!end_q7n
```

Run it:

```bash
cd /Users/stuart/repos/tantivy/stuart/research/algolia_test_1
node critical_test_v2.js
```