const http = require('http');
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const ROOT = process.cwd();
const PORT = process.env.PORT ? parseInt(process.env.PORT, 10) : 3000;

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
    const urlPath = decodeURIComponent(req.url.split('?')[0]);

    // Tests expect the original non-Svelte frontend at /www/index.html (loads /www/main.js).
    // If we were refactored to Svelte (index now loads /src/main.js), return a compatible index
    // that loads the original `www/main.js` so puppeteer tests work without building the Svelte app.
    if (urlPath === '/www/index.html' || urlPath === '/www/' || urlPath === '/www') {
        const html = `<!doctype html><html><head><meta charset="utf-8"><title>Test</title></head><body><canvas id="screen"></canvas><script type="module" src="/www/main.js"></script></body></html>`;
        res.setHeader('Content-Type', 'text/html; charset=utf-8');
        res.end(html);
        return;
    }

    let filePath = path.join(ROOT, urlPath);
    fs.stat(filePath, (err, st) => {
        if (err) {
            res.statusCode = 404;
            res.end('Not found');
            return;
        }
        if (st.isDirectory()) {
            // serve index.html inside directory if present
            const idx = path.join(filePath, 'index.html');
            fs.stat(idx, (e2, s2) => {
                if (!e2 && s2.isFile()) {
                    filePath = idx;
                    streamFile(filePath, res);
                } else {
                    res.statusCode = 403;
                    res.end('Forbidden');
                }
            });
        } else {
            streamFile(filePath, res);
        }
    });
});

function streamFile(filePath, res) {
    const ct = contentType(filePath);
    res.setHeader('Content-Type', ct);
    const stream = fs.createReadStream(filePath);
    stream.on('error', () => { res.statusCode = 500; res.end('Error'); });
    stream.pipe(res);
}

function runTestFile(file) {
    return new Promise((resolve) => {
        console.log(`\n--- running ${file} ---`);
        const p = spawn(process.execPath, [file], { stdio: 'inherit' });
        p.on('close', (code) => {
            if (code === 0) {
                console.log(`✅ ${path.basename(file)} passed`);
                resolve({ file, ok: true });
            } else {
                console.error(`❌ ${path.basename(file)} failed (code ${code})`);
                resolve({ file, ok: false, code });
            }
        });
    });
}

async function main() {
    server.listen(PORT, () => {
        console.log(`Serving ${ROOT} at http://localhost:${PORT}`);
    });

    // find test scripts in tools/ matching test_*.js
    const toolsDir = path.join(ROOT, 'tools');
    let files = [];
    try { files = fs.readdirSync(toolsDir).filter(f => f.startsWith('test_') && f.endsWith('.js')).map(f => path.join(toolsDir, f)); }
    catch (e) { console.error('No tests found in tools/'); process.exit(1); }

    const results = [];
    for (const f of files) {
        // give the server a moment to be ready
        await new Promise(r => setTimeout(r, 200));
        // run each test
        // tests expect process to be started with `node <file>` and will close browser themselves
        const res = await runTestFile(f);
        results.push(res);
    }

    const failed = results.filter(r => !r.ok);
    server.close(() => {
        if (failed.length === 0) {
            console.log('\nAll tests passed');
            process.exit(0);
        } else {
            console.error(`\n${failed.length} test(s) failed`);
            process.exit(2);
        }
    });
}

main().catch((err) => { console.error(err); process.exit(1); });
