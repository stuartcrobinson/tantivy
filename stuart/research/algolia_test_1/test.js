import { algoliasearch } from 'algoliasearch';
import dotenv from 'dotenv';

dotenv.config();

const client = algoliasearch(
  process.env.ALGOLIA_APP_ID,
  process.env.ALGOLIA_ADMIN_KEY
);

const indexName = 'data_model_test';

async function test() {
  console.log('=== Algolia Data Model & Query Tests ===\n');
  
  await client.saveObjects({
    indexName,
    objects: [{
      objectID: '1',
      title: 'Gaming Laptop',
      price: 1200,
      nested: { deep: { value: 'test' } }
    }]
  });
  
  await new Promise(r => setTimeout(r, 2000));
  
  console.log('Test 1: Prefix search "gam"');
  const r1 = await client.search({ requests: [{ indexName, query: 'gam' }] });
  console.log(`  Found ${r1.results[0].hits.length} hits`);
  console.log(`  Matches: ${r1.results[0].hits.map(h => h.title).join(', ')}\n`);
  
  console.log('Test 2: Multi-word "gaming laptop"');
  const r2 = await client.search({ requests: [{ indexName, query: 'gaming laptop' }] });
  console.log(`  Found ${r2.results[0].hits.length} hits\n`);
  
  console.log('Test 3: Typo "gamng"');
  const r3 = await client.search({ requests: [{ indexName, query: 'gamng' }] });
  console.log(`  Found ${r3.results[0].hits.length} hits\n`);
  
  console.log('Test 4: Search nested.deep.value');
  const r4 = await client.search({ requests: [{ indexName, query: 'test' }] });
  console.log(`  Found ${r4.results[0].hits.length} hits`);
  console.log(`  Searches all fields by default (no path needed)\n`);
  
  console.log('Test 5: Restrict to specific fields');
  await client.setSettings({
    indexName,
    indexSettings: { searchableAttributes: ['title'] }
  });
  await new Promise(r => setTimeout(r, 2000));
  
  const r5a = await client.search({ requests: [{ indexName, query: 'gaming' }] });
  const r5b = await client.search({ requests: [{ indexName, query: 'test' }] });
  console.log(`  "gaming" in title → ${r5a.results[0].hits.length} hits`);
  console.log(`  "test" (nested field, not searchable) → ${r5b.results[0].hits.length} hits`);
}

test().catch(console.error);