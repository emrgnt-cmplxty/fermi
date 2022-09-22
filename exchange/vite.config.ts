import inject from '@rollup/plugin-inject'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import tsconfigPaths from 'vite-tsconfig-paths'

import { getAugmentedThemes, THEME_KEYS } from './src/theme'
import getKeys from './src/utils/sassVariableInjector'

const augmentedThemes = getAugmentedThemes()

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [tsconfigPaths(), react()],
  resolve: {
    alias: {
      // for algorand sdk
      path: 'path-browserify',
    },
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV),
  },
  css: {
    preprocessorOptions: {
      scss: {
        includePaths: ['./src/styles'],
        functions: {
          'get($keys)': getKeys(augmentedThemes),
        },
        additionalData: '$themes: ' + THEME_KEYS.join(', ') + ';',
      },
    },
  },
})
