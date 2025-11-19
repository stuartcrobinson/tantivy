import { algoliasearch } from 'algoliasearch';
import dotenv from 'dotenv';

dotenv.config();

const client = algoliasearch(
  process.env.ALGOLIA_APP_ID,
  process.env.ALGOLIA_ADMIN_KEY
);

const indexName = 'prefix_behavior_test';

async function test() {
  console.log('=== CRITICAL: Multi-Word Prefix Behavior ===\n');

  try {
    await client.deleteIndex({ indexName });
    console.log('Deleted existing index');
    await new Promise(r => setTimeout(r, 3000));
  } catch (e) {
    console.log('No existing index to delete');
  }

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

  console.log('Waiting for indexing...');
  await new Promise(r => setTimeout(r, 3000));

  console.log('\nDocument: {"title": "Gaming Laptop"}');
  console.log('Question: Does prefix "lap" match?\n');

  const result = await client.search({
    requests: [{ indexName, query: 'lap' }]
  });

  const hits = result.results[0].hits;
  console.log(`Query "lap": ${hits.length} hits`);

  if (hits.length > 0) {
    console.log('Matched documents:');
    hits.forEach(h => console.log(`  - ${h.title}`));
    console.log('\n✅ CONFIRMED: Algolia does per-word prefix matching');
    console.log('   "lap" matches second word in "Gaming Laptop"');
    console.log('   → Flapjack MUST implement EdgeNgramFilter\n');
  } else {
    console.log('❌ NO MATCH: Algolia does NOT match mid-string prefixes');
    console.log('   → Can use simpler NgramTokenizer (full-text)\n');
  }

  console.log('Test 2: Does "gam" match?');
  const r2 = await client.search({
    requests: [{ indexName, query: 'gam' }]
  });
  console.log(`Query "gam": ${r2.results[0].hits.length} hits`);
  console.log(`  Matches: ${r2.results[0].hits.map(h => h.title).join(', ')}`);

  console.log('\nTest 3: Does "mou" match "Gaming Mouse"?');
  const r3 = await client.search({
    requests: [{ indexName, query: 'mou' }]
  });
  console.log(`Query "mou": ${r3.results[0].hits.length} hits`);
  console.log(`  Matches: ${r3.results[0].hits.map(h => h.title).join(', ')}`);

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
}



test().catch(console.error);