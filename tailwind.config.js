/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: ['class', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        'cyber-primary': 'var(--accent-primary)',
        'cyber-hover': 'var(--accent-hover)',
        'cyber-accent': 'var(--text-accent)',
        'cyber-success': 'var(--success-primary)',
        'cyber-warning': 'var(--warning-primary)',
        'cyber-error': 'var(--error-primary)',
      },
      boxShadow: {
        'glow-sm': 'var(--glow-sm)',
        'glow-md': 'var(--glow-md)',
        'glow-lg': 'var(--glow-lg)',
        'cyber': 'var(--shadow-glow)',
      },
      backgroundImage: {
        'cyber-gradient': 'linear-gradient(135deg, var(--accent-primary), var(--accent-hover))',
        'glass': 'var(--bg-glass)',
      },
      animation: {
        'pulse-glow': 'pulse-glow 2s ease-in-out infinite',
        'scan': 'scan 3s ease-in-out infinite',
      },
      keyframes: {
        'pulse-glow': {
          '0%, 100%': { opacity: 1, boxShadow: 'var(--glow-sm)' },
          '50%': { opacity: 0.8, boxShadow: 'var(--glow-md)' },
        },
        'scan': {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(100%)' },
        },
      },
    },
  },
  plugins: [],
}
