const puppeteer = require('puppeteer');
const fs = require('fs');

(async () => {
    const testId = '002-turn_sequence';
    const url = 'http://localhost:3000/www/index.html';
    const outDir = `screenshots/${testId}`;
    if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });

    const browser = await puppeteer.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
    const page = await browser.newPage();
    await page.setViewport({ width: 960, height: 600 });

    console.log('loading', url);
    await page.goto(url, { waitUntil: 'networkidle2' });

    // wait for canvas to be present and for wasm to initialize (canvas sized)
    await page.waitForSelector('#screen');
    await page.waitForFunction(() => {
        const c = document.getElementById('screen');
        return c && c.width > 0 && c.height > 0;
    }, { timeout: 5000 });

    const trace = [];
    let shotCounter = 0;
    const testStartTime = Date.now();
    async function capture(label) {
        shotCounter += 1;
        const timestamp = Date.now() - testStartTime;
        const state = await page.evaluate(() => {
            const c = document.getElementById('screen');
            return {
                dataUrl: c.toDataURL('image/png'),
                pos_x: window.wasm.get_pos_x(),
                pos_y: window.wasm.get_pos_y(),
                dir_x: window.wasm.get_dir_x(),
                dir_y: window.wasm.get_dir_y(),
                plane_x: window.wasm.get_plane_x(),
                plane_y: window.wasm.get_plane_y()
            };
        });

        const checkpoint = {
            id: shotCounter,
            timestamp_ms: timestamp,
            label: label || `checkpoint_${shotCounter}`,
            position: { x: state.pos_x, y: state.pos_y },
            direction: { x: state.dir_x, y: state.dir_y },
            plane: { x: state.plane_x, y: state.plane_y },
            angle_deg: Math.atan2(state.dir_y, state.dir_x) * 180 / Math.PI
        };
        trace.push(checkpoint);

        const base64Data = state.dataUrl.replace(/^data:image\/png;base64,/, '');
        const buffer = Buffer.from(base64Data, 'base64');
        const file = `${outDir}/${testId}-${String(shotCounter).padStart(3, '0')}.png`;
        fs.writeFileSync(file, buffer);
        console.log(`📸 [${shotCounter}] ${checkpoint.label}: pos=(${state.pos_x.toFixed(2)}, ${state.pos_y.toFixed(2)}) dir=(${state.dir_x.toFixed(2)}, ${state.dir_y.toFixed(2)}) angle=${checkpoint.angle_deg.toFixed(1)}°`);
    }

    const sleep = (ms) => new Promise((res) => setTimeout(res, ms));

    // 1) initial screenshot
    await sleep(300);
    await capture('initial_state');

    // 2-10) nine small right turns of 0.05s, capturing after each
    for (let i = 1; i <= 9; i++) {
        await page.keyboard.down('ArrowRight');
        await sleep(50);
        await page.keyboard.up('ArrowRight');
        await sleep(50);
        await capture(`turn_${i}_0.05s`);
    }

    // Save trace data
    const traceFile = `${outDir}/${testId}-trace.json`;
    fs.writeFileSync(traceFile, JSON.stringify(trace, null, 2));
    console.log('\n📊 Trace saved to', traceFile);

    await browser.close();
    console.log('✅ done', testId);
})();
