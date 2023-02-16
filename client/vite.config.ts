import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    minify: 'terser',
    terserOptions: {
      ecma: 2020,
      compress: {
        passes: 4
      }
    }
  },
  worker: {
    format: 'es'
  },
  envDir: '..',
  envPrefix: 'CFN_',
});
