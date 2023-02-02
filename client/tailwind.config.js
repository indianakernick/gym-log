/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{css,vue}",
  ],
  theme: {
    extend: {
      gridTemplateColumns: {
        'header': '1fr auto 1fr'
      }
    },
  },
  plugins: [],
}
