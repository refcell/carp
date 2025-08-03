// Test the API endpoints
const baseUrl = 'http://localhost:3000/api';

async function testEndpoint(name, url) {
  console.log(`\nTesting ${name}:`);
  try {
    const response = await fetch(url);
    console.log('Status:', response.status);
    const text = await response.text();
    console.log('Response length:', text.length);
    
    if (response.ok) {
      try {
        const data = JSON.parse(text);
        console.log('Parsed data:', JSON.stringify(data, null, 2).substring(0, 500));
      } catch (e) {
        console.log('Failed to parse JSON:', e.message);
        console.log('Raw response:', text.substring(0, 200));
      }
    } else {
      console.log('Error response:', text);
    }
  } catch (error) {
    console.log('Fetch error:', error.message);
  }
}

async function runTests() {
  await testEndpoint('Latest Agents', `${baseUrl}/v1/agents/latest?limit=5`);
  await testEndpoint('Trending Agents', `${baseUrl}/v1/agents/trending?limit=5`);
}

runTests();