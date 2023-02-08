/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{css,vue}",
  ],
  theme: {
    extend: {},
    transitionDuration: {
      DEFAULT: '200ms'
    }
  },
  plugins: [],
}
