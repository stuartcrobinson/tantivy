=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:12:14.677 pm
---------------------
q5m ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/research/algolia_test_1/critical_test_v2.js
w8n ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/research/algolia_test_1/critical_test_v2.js
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: q5m]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/research/algolia_test_1/critical_test_v2.js"
old_text = <<'EOT_q5m'
  console.log('Creating index and adding documents...');
  await client.batch({
    indexName,
    batchWriteParams: {
      requests: [
        { action: 'addObject', body: { objectID: '1', title: 'Gaming Laptop' } },
        { action: 'addObject', body: { objectID: '2', title: 'Laptop Stand' } },
        { action: 'addObject', body: { objectID: '3', title: 'Gaming Mouse' } }
      ]
    }
  });
EOT_q5m
new_text = <<'EOT_q5m'
  console.log('Creating index and adding documents...');
  await client.batch({
    indexName,
    batchWriteParams: {
      requests: [
        { action: 'addObject', body: { objectID: '1', title: 'Gaming Laptop' } },
        { action: 'addObject', body: { objectID: '2', title: 'Laptop Stand' } },
        { action: 'addObject', body: { objectID: '3', title: 'Gaming Mouse' } },
        { action: 'addObject', body: { objectID: '4', title: 'La' } }
      ]
    }
  });
EOT_q5m
#!end_q5m
```

```sh nesl
#!nesl [@three-char-SHA-256: w8n]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/research/algolia_test_1/critical_test_v2.js"
old_text = <<'EOT_w8n'
  console.log('\nTest 4: Does "la" (incomplete prefix) match "Laptop"?');
  const result4 = await client.search({
    requests: [{ indexName, query: 'la' }]
  });
  console.log(`Query "la": ${result4.results[0].hits.length} hits`);
  if (result4.results[0].hits.length > 0) {
    console.log('  Matches:', result4.results[0].hits.map(h => h.title).join(', '));
  }
}
EOT_w8n
new_text = <<'EOT_w8n'
  console.log('\nTest 4: Does "la" (incomplete prefix) match "Laptop"?');
  const result4 = await client.search({
    requests: [{ indexName, query: 'la' }]
  });
  console.log(`Query "la": ${result4.results[0].hits.length} hits`);
  if (result4.results[0].hits.length > 0) {
    console.log('  Matches:', result4.results[0].hits.map(h => h.title).join(', '));
  }

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
}
EOT_w8n
#!end_w8n
```

Run it:

```bash
cd stuart/research/algolia_test_1/
node critical_test_v2.js
```