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

    let shotCounter = 0;
    async function capture() {
        shotCounter += 1;
        const dataUrl = await page.evaluate(() => {
            const c = document.getElementById('screen');
            return c.toDataURL('image/png');
        });
        const base64Data = dataUrl.replace(/^data:image\/png;base64,/, '');
        const buffer = Buffer.from(base64Data, 'base64');
        const file = `${outDir}/${testId}-${String(shotCounter).padStart(3, '0')}.png`;
        fs.writeFileSync(file, buffer);
        console.log('saved', file);
    }

    const sleep = (ms) => new Promise((res) => setTimeout(res, ms));

    // 1) initial screenshot
    await sleep(300);
    await capture();

    // 2-10) nine small right turns of 0.05s, capturing after each
    for (let i = 1; i <= 9; i++) {
        await page.keyboard.down('ArrowRight');
        await sleep(50);
        await page.keyboard.up('ArrowRight');
        await sleep(50);
        await capture();
    }

    await browser.close();
    console.log('done', testId);
})();
