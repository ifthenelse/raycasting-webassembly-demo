# SEO, image SEO and AI SEO audit

Audited: 2026-10-16. Site: https://ifthenelse.github.io/raycasting-webassembly-demo/ (one page, deployed by
`.github/workflows/pages.yml`). No Search Console or analytics data was available, so this covers technical and
on-page checks only.

## Summary

Before this change the site could not rank. It had no build on Pages, and the production bundle failed to load
its textures because they were referenced as plain string paths that Vite never bundles. The page was an empty
`<div id="app">` with a canvas: no description, no heading, no indexable text, and its favicon returned 404.

All of this is fixed in this change. What remains is outside the code: domain authority and backlinks.

## Technical SEO

| Issue                                                                                                     | Impact                     | Fix (status)                                                                                                                                                |
| --------------------------------------------------------------------------------------------------------- | -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| No public deployment                                                                                      | Critical                   | GitHub Pages workflow builds the wasm crate and the site on every push to `main` (done)                                                                     |
| Textures 404 in production build (`'../../assets/x.png'` strings)                                         | Critical                   | `new URL(..., import.meta.url)` so Vite bundles them (done)                                                                                                 |
| Absolute asset base breaks under the `/raycasting-webassembly-demo/` subpath                              | Critical                   | `base: './'` in `vite.config.mjs` (done)                                                                                                                    |
| No `viewport` meta                                                                                        | High                       | Added (done)                                                                                                                                                |
| No canonical                                                                                              | Medium                     | Self-referencing canonical (done)                                                                                                                           |
| `favicon.png` 404                                                                                         | Low                        | `public/favicon.png` (done)                                                                                                                                 |
| No sitemap                                                                                                | Medium                     | `public/sitemap.xml` with image extension (done). Submit it in Search Console.                                                                              |
| `robots.txt` on a project subpath is not read by crawlers (only `ifthenelse.github.io/robots.txt` counts) | Low                        | Shipped anyway as a harmless pointer. Nothing blocks crawling. Use a custom domain if crawl rules ever matter.                                              |
| ~8 MB of textures load before the first frame                                                             | Medium (LCP of the canvas) | Lossless WebP, previously 12.3 MB PNG (done). Smaller textures (e.g. 512×512) would cut this much further but change the look, so that is the owner's call. |

## On-page SEO

| Element          | Before                        | After                                                                                                                                          |
| ---------------- | ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Title            | "Super Simple WASM Raycaster" | "WebAssembly Raycaster in Rust – Play in Your Browser" (52 chars)                                                                              |
| Meta description | none                          | 175-char summary naming Rust, WebAssembly, the feature set and 320×200. It may be truncated at around 160 chars, but the key terms come first. |
| H1               | none                          | "WebAssembly Raycaster in Rust"                                                                                                                |
| Body text        | 0 words                       | About 280 words: controls, features, how raycasting works, why Rust + WASM, source link                                                        |
| Structured data  | none                          | `WebApplication` + `VideoGame`, `SoftwareSourceCode`, `FAQPage` (JSON-LD, static in HTML)                                                      |
| Social           | none                          | Open Graph + Twitter `summary_large_image` with a 1200×630 screenshot                                                                          |

Target queries: "webassembly raycaster", "rust raycaster", "raycasting in rust wasm", "wolfenstein raycaster
browser", "wasm-bindgen canvas framebuffer".

## Image SEO

Image SEO score: 3/10 before, 8/10 after.

| Image                                                        | Issues                                                                              | Fix                                                                                                      |
| ------------------------------------------------------------ | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Textures (`wall`, `floor`, `ceiling`, `skybox`, `lamp-wall`) | 1536×1024 PNG, 2.4–2.7 MB each; loaded by JS into WASM, so they are never indexable | Lossless WebP (−34%, pixel-identical). No alt text needed: these images never reach the DOM.             |
| Game canvas                                                  | Not an image; invisible to Google Images                                            | Static `og-image.jpg` screenshot (132 KB) referenced from OG tags, JSON-LD `image` and the image sitemap |
| `og-image.jpg`                                               | (new)                                                                               | Descriptive `og:image:alt`, explicit width and height                                                    |
| Favicon                                                      | 404                                                                                 | 48×48 PNG cropped from the wall texture                                                                  |

Expected Core Web Vitals impact: about 4 MB less to download before the first rendered frame. There is no CLS
risk: the canvas is sized by JS before the first paint, and the text section sits below it.

Open item: the original PNGs in `assets/` are no longer referenced. Delete them to keep the repo small
(left in place pending approval).

## AI SEO (ChatGPT, Perplexity, Claude, Google AI Overviews)

- `llms.txt` gives a plain-text summary: what it is, how it works, controls, links.
- The page now has extractable answer blocks: "How does a raycaster work?" and "Why Rust and WebAssembly?"
  Each is 40–60 words, sits under a question-shaped H2 and is mirrored in `FAQPage` schema.
- The content is static HTML, so agents that do not run JS can still read it.
- No crawler is blocked. Discovery and training bots are all allowed, which suits an open-source demo.
- The biggest lever is presence, not markup. AI engines cite where demos get discussed. Post the live link to
  r/rust, r/webassembly, Hacker News (Show HN) and the repo's GitHub topics (`raycasting`, `webassembly`, `rust`,
  `wasm-bindgen`, `game-engine`), and set the repo's website field to the Pages URL.

## Programmatic SEO

Not recommended for this site. Programmatic SEO needs a dataset with real search demand and unique value per
page. This project has one demo and no such dataset. Template pages ("raycaster in X", one page per random map
seed) would be thin, doorway-style content and would hurt the main page.

If the site grows, the one playbook that would fit is a glossary or tutorial cluster built from the engine itself:
DDA, fish-eye correction, floor casting, sprite billboarding, bilinear sampling. Each page would show the real
code from `src/lib.rs` and a live toggle. That is maybe 5–8 hand-written pages, not a generated set.

## Owner actions

1. Repo Settings → Pages → Source: **GitHub Actions** (required once, or the deploy job fails).
2. Verify the site in Google Search Console and submit `sitemap.xml`.
3. Set the GitHub repo "Website" field and topics.
