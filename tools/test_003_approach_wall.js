const puppeteer = require('puppeteer');
const fs = require('fs');

(async () => {
    const testId = '003-approach_wall';
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

    // a) start the game and take a screenshot
    await sleep(300);
    await capture();

    // b) move straight for 0.25s and take a screenshot
    await page.keyboard.down('ArrowUp');
    await sleep(250);
    await page.keyboard.up('ArrowUp');
    await sleep(100);
    await capture();

    // c) move straight for 0.25s and turn 75 degrees clockwise and take a third screenshot
    // move straight
    await page.keyboard.down('ArrowUp');
    await sleep(250);
    // rotate precisely by 75 degrees (clockwise = negative radians)
    const desiredDeg = 75;
    const desiredRad = -(desiredDeg * Math.PI / 180.0);
    // dispatch a CustomEvent that main.js listens to and performs the rotation
    await page.evaluate((r) => window.dispatchEvent(new CustomEvent('rotate_by', { detail: r })), desiredRad);
    await page.keyboard.up('ArrowUp');
    await sleep(100);
    await capture();

    await browser.close();
    console.log('done', testId);
})();
