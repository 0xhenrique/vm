// Node.js HTTP Server for Benchmarking
// Usage: node http_server.js [port]

const http = require('http');

const PORT = process.argv[2] || 8080;
const RESPONSE = 'Hello from Node.js!';

const server = http.createServer((req, res) => {
  res.writeHead(200, {
    'Content-Type': 'text/plain',
    'Content-Length': Buffer.byteLength(RESPONSE)
  });
  res.end(RESPONSE);
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`Node.js server listening on port ${PORT}`);
});
