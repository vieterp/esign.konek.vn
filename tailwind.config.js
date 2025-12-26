/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "media",
  theme: {
    extend: {
      colors: {
        // KONEK Brand Colors
        // Primary - Navy (Trust, Foundation)
        navy: {
          50: '#E8EDF4',
          100: '#C5D3E8',
          200: '#A2B9DC',
          300: '#5B7BA3',
          400: '#3D5F89',
          500: '#2D4A6F',
          600: '#234058',
          700: '#1B365D', // Primary brand color
          800: '#152B4A',
          900: '#0F1F38',
        },
        // Secondary - Ocean Blue (Growth, Progress)
        ocean: {
          50: '#EBF4FC',
          100: '#C9E0F5',
          200: '#A7CCEE',
          300: '#85B8E7',
          400: '#63A4E0',
          500: '#4A90D9', // Secondary brand color
          600: '#3A7AC4',
          700: '#2E6DB5',
          800: '#235A96',
          900: '#1A4777',
        },
        // Accent - Sky Blue (Achievement, Success)
        sky: {
          50: '#F0F7FC',
          100: '#D9EBF7',
          200: '#B3D7EF',
          300: '#8DC3E7',
          400: '#7CB9E8', // Accent brand color
          500: '#5BA5DE',
          600: '#4891D4',
          700: '#357DCA',
        },
        // Semantic aliases for easier usage
        primary: {
          50: '#E8EDF4',
          100: '#C5D3E8',
          200: '#A2B9DC',
          300: '#5B7BA3',
          400: '#3D5F89',
          500: '#2D4A6F',
          600: '#234058',
          700: '#1B365D',
          800: '#152B4A',
          900: '#0F1F38',
          DEFAULT: '#1B365D',
        },
      },
      fontFamily: {
        sans: ['Be Vietnam Pro', 'system-ui', '-apple-system', 'sans-serif'],
        primary: ['Be Vietnam Pro', 'sans-serif'],
      },
      fontSize: {
        'hero': ['3.5rem', { lineHeight: '1.2', fontWeight: '700' }],
        'h1': ['2.5rem', { lineHeight: '1.2', fontWeight: '700' }],
        'h2': ['2rem', { lineHeight: '1.3', fontWeight: '600' }],
        'h3': ['1.5rem', { lineHeight: '1.4', fontWeight: '600' }],
      },
    },
  },
  plugins: [],
};
