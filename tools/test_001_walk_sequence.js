const puppeteer = require('puppeteer');
const fs = require('fs');

(async () => {
    const testId = '001-walk_sequence';
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

    // initial state
    await sleep(300);
    await capture();

    // turn right for 0.05s
    await page.keyboard.down('ArrowRight');
    await sleep(50);
    await page.keyboard.up('ArrowRight');
    await sleep(50);

    // walk straight for 0.1s and take a screenshot
    await page.keyboard.down('ArrowUp');
    await sleep(100);
    await page.keyboard.up('ArrowUp');
    await sleep(100);
    await capture();

    // walk straight for 0.1s and take another screenshot
    await page.keyboard.down('ArrowUp');
    await sleep(100);
    await page.keyboard.up('ArrowUp');
    await sleep(100);
    await capture();

    // turn right for 0.1s
    await page.keyboard.down('ArrowRight');
    await sleep(100);
    await page.keyboard.up('ArrowRight');
    await sleep(50);

    // walk straight for 0.1s and take another screenshot
    await page.keyboard.down('ArrowUp');
    await sleep(100);
    await page.keyboard.up('ArrowUp');
    await sleep(100);
    await capture();

    // turn right for 1s
    await page.keyboard.down('ArrowRight');
    await sleep(1000);
    await page.keyboard.up('ArrowRight');
    await sleep(100);

    // walk straight for 0.1s and take final screenshot
    await page.keyboard.down('ArrowUp');
    await sleep(100);
    await page.keyboard.up('ArrowUp');
    await sleep(100);
    await capture();

    await browser.close();
    console.log('done', testId);
})();
