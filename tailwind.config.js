/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'neon-purple': '#bc13fe',
        'neon-yellow': '#faff00',
        'bunker-black': '#050505',
      },
      boxShadow: {
        'neon-p': '0 0 10px #bc13fe, 0 0 20px #bc13fe',
        'neon-y': '0 0 10px #faff00, 0 0 20px #faff00',
      }
    },
  },
  plugins: [],
}
