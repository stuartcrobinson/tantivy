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
  
  await client.deleteIndex({ indexName }).catch(() => {});
  await new Promise(r => setTimeout(r, 1000));
  
  await client.saveObjects({
    indexName,
    objects: [
      { objectID: '1', title: 'Gaming Laptop' },
      { objectID: '2', title: 'Laptop Stand' },
      { objectID: '3', title: 'Gaming Mouse' }
    ]
  });
  
  await new Promise(r => setTimeout(r, 2000));
  
  console.log('Document: {"title": "Gaming Laptop"}');
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
}

test().catch(console.error);