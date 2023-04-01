import type { Config } from 'tailwindcss';

export default {
  content: [
    './index.html',
    './src/**/*.{css,vue}',
  ],
  theme: {
    extend: {},
    transitionDuration: {
      DEFAULT: '250ms'
    }
  },
  plugins: [],
} satisfies Config;
