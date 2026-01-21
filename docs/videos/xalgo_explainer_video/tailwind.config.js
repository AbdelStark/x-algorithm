/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        'x-blue': '#1D9BF0',
        'x-gold': '#FFD93D',
        'x-green': '#00BA7C',
        'x-pink': '#F91880',
        'x-dark': '#0D1117',
        'x-gray': '#8B949E',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [],
};
