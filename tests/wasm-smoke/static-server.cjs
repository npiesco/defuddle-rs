const http = require('http');
const fs = require('fs');
const path = require('path');

const port = Number(process.argv[2] || 4173);
const root = path.resolve(process.cwd(), process.argv[3] || '.');
const mime = {
  '.css': 'text/css; charset=utf-8',
  '.d.ts': 'text/plain; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.wasm': 'application/wasm',
};

function resolvePath(urlPath) {
  const cleanPath = decodeURIComponent((urlPath || '/').split('?')[0]);
  const normalized = path.normalize(cleanPath).replace(/^([.][.][\\/])+/, '');
  let filePath = path.join(root, normalized === path.sep ? 'tests/wasm-smoke/index.html' : normalized.replace(/^[/\\]/, ''));

  if (fs.existsSync(filePath) && fs.statSync(filePath).isDirectory()) {
    filePath = path.join(filePath, 'index.html');
  }

  return filePath;
}

const server = http.createServer((req, res) => {
  const filePath = resolvePath(req.url);
  fs.readFile(filePath, (error, data) => {
    if (error) {
      res.statusCode = 404;
      res.end('not found');
      return;
    }

    res.setHeader('Content-Type', mime[path.extname(filePath)] || 'application/octet-stream');
    res.end(data);
  });
});

server.listen(port, '127.0.0.1', () => {
  console.log(`defuddle static smoke server listening on http://127.0.0.1:${port}/`);
});
