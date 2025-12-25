const puppeteer = require('puppeteer');

(async () => {
    const url = process.argv[2] || 'http://localhost:5174/';
    console.log('Checking', url);
    const browser = await puppeteer.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
    const page = await browser.newPage();

    page.on('console', msg => {
        const args = msg.args();
        Promise.all(args.map(a => a.jsonValue())).then(vals => {
            console.log('PAGE LOG:', msg.type(), ...vals);
        }).catch(() => {
            console.log('PAGE LOG:', msg.type(), msg.text());
        });
    });

    page.on('pageerror', err => console.log('PAGE ERROR:', err && err.toString()));

    try {
        await page.goto(url, { waitUntil: 'networkidle2', timeout: 10000 });
        // wait a little to capture runtime console messages
        await page.waitForTimeout(1500);
    } catch (e) {
        console.error('Puppeteer error', e && e.toString());
    }

    await browser.close();
    process.exit(0);
})();
