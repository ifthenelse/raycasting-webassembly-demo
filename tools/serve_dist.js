const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = process.env.PORT ? parseInt(process.env.PORT, 10) : 5174;
const ROOT = path.join(process.cwd(), 'dist');

function contentType(file) {
    const ext = path.extname(file).toLowerCase();
    switch (ext) {
        case '.html': return 'text/html; charset=utf-8';
        case '.js': return 'application/javascript; charset=utf-8';
        case '.css': return 'text/css; charset=utf-8';
        case '.png': return 'image/png';
        case '.jpg': case '.jpeg': return 'image/jpeg';
        case '.wasm': return 'application/wasm';
        case '.json': return 'application/json; charset=utf-8';
        default: return 'application/octet-stream';
    }
}

const server = http.createServer((req, res) => {
    let urlPath = decodeURIComponent(req.url.split('?')[0]);
    if (urlPath === '/') urlPath = '/index.html';
    const filePath = path.join(ROOT, urlPath);
    fs.stat(filePath, (err, st) => {
        if (err || !st.isFile()) {
            res.statusCode = 404; res.end('Not found'); return;
        }
        res.setHeader('Content-Type', contentType(filePath));
        fs.createReadStream(filePath).pipe(res);
    });
});

server.listen(PORT, () => {
    console.log(`Serving dist at http://localhost:${PORT}`);
});

process.on('SIGINT', () => {
    server.close(() => process.exit(0));
});
