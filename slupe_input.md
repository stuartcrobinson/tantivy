=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 03:59:33.668 pm
---------------------
k2p ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/research/algolia_test_1/critical_test.js
=== END ===
# Index Creation Race Condition

```sh nesl
#!nesl [@three-char-SHA-256: k2p]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/research/algolia_test_1/critical_test.js"
old_text = <<'EOT_k2p'
  await client.deleteIndex({ indexName }).catch(() => {});
  await new Promise(r => setTimeout(r, 1000));
  
  await client.saveObjects({
EOT_k2p
new_text = <<'EOT_k2p'
  await client.deleteIndex({ indexName }).catch(() => {});
  await new Promise(r => setTimeout(r, 2000));
  
  await client.saveObjects({
EOT_k2p
#!end_k2p
```

```bash
node critical_test.js
```

Increased wait after delete. If still fails, index may not auto-create on saveObjects (API version difference). Share output.