const puppeteer = require('puppeteer');

(async () => {
  const url = 'http://localhost:3000/www/index.html';
  const browser = await puppeteer.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
  const page = await browser.newPage();
  await page.goto(url, { waitUntil: 'networkidle2' });

  await page.waitForSelector('#screen');
  await page.waitForFunction(
    () => {
      const c = document.getElementById('screen');
      return c && c.width > 0 && c.height > 0;
    },
    { timeout: 5000 }
  );

  // helper to sample arrays in the wasm instance by importing the pkg inside the page
  const sample = await page.evaluate(async () => {
    const init = (await import('/pkg/wasm_raycaster.js')).default;
    const wasm = await init();
    wasm.init();

    const w = wasm.get_width();
    const h = wasm.get_height();
    const mem = wasm.memory.buffer;
    const fbPtr = wasm.get_framebuffer_ptr();
    const matPtr = wasm.get_material_ptr();
    const tuPtr = wasm.get_tex_u_ptr();
    const tvPtr = wasm.get_tex_v_ptr();

    const materials = new Uint8Array(mem, matPtr, w * h);
    const texU = new Uint16Array(mem, tuPtr, w * h);
    const texV = new Uint16Array(mem, tvPtr, w * h);

    // sample center column rows
    const cx = Math.floor(w / 2);
    const rows = [];
    for (let dy = -10; dy <= 10; dy++) {
      const y = Math.floor(h / 2) + dy;
      if (y < 0 || y >= h) continue;
      const idx = y * w + cx;
      rows.push({ y, mat: materials[idx], u: texU[idx], v: texV[idx] });
    }

    // sample across a horizontal row (middle of screen)
    const sampleY = Math.floor(h / 2);
    const cols = [];
    for (let x = cx - 40; x <= cx + 40; x++) {
      if (x < 0 || x >= w) continue;
      const idx = sampleY * w + x;
      cols.push({ x, mat: materials[idx], u: texU[idx], v: texV[idx] });
    }

    // Now simulate a small forward step and sample again
    wasm.step(1, 0, 0, 0); // small step
    const rows2 = [];
    for (let dy = -10; dy <= 10; dy++) {
      const y = Math.floor(h / 2) + dy;
      if (y < 0 || y >= h) continue;
      const idx = y * w + cx;
      rows2.push({ y, mat: materials[idx], u: texU[idx], v: texV[idx] });
    }

    const cols2 = [];
    for (let x = cx - 40; x <= cx + 40; x++) {
      if (x < 0 || x >= w) continue;
      const idx = sampleY * w + x;
      cols2.push({ x, mat: materials[idx], u: texU[idx], v: texV[idx] });
    }

    return { rows_before: rows, cols_before: cols, rows_after_step: rows2, cols_after_step: cols2 };
  });

  console.log(JSON.stringify(sample, null, 2));
  await browser.close();
})();
