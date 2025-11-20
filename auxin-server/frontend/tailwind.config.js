/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#fef2f2',
          100: '#fee2e2',
          200: '#fecaca',
          300: '#fca5a5',
          400: '#f87171',
          500: '#b91c3b',
          600: '#991b35',
          700: '#7a162a',
          800: '#5b1020',
          900: '#3c0b15',
          950: '#1d050b',
        },
      },
    },
  },
  plugins: [],
}
