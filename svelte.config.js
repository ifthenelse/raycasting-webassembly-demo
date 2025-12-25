const { vitePreprocess } = require('@sveltejs/vite-plugin-svelte');

module.exports = {
  preprocess: vitePreprocess(),
  compilerOptions: {
    runes: false,
  },
};
