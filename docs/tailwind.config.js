/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: ['./index.html'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Geist', 'system-ui', 'sans-serif'],
        mono: ['"Geist Mono"', 'ui-monospace', 'monospace'],
      },
      colors: {
        ink: {
          900: '#17140F',
          800: '#1F1A12',
          700: '#2A2217',
          600: '#352B1E',
          500: '#4A3D2C',
          400: '#78705A',
          300: '#B5A889',
          200: '#E8DEC2',
          100: '#F4ECD8',
        },
        cream: {
          100: '#FFFBF1',
          200: '#FAF6EC',
          300: '#F0E8D4',
          400: '#E4D8B8',
          500: '#C8B88E',
        },
        honey: {
          DEFAULT: '#D4A94E',
          light: '#E6B75B',
          dark: '#A8841F',
          subtle: '#F5E9C3',
        },
        sage: '#7A9468',
        terracotta: '#C8644A',
      },
    },
  },
};
