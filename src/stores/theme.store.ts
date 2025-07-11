import { createSignal, createEffect } from 'solid-js';

export type Theme = 'light' | 'dark' | 'auto';

// Available DaisyUI themes
export const AVAILABLE_THEMES = [
  'light',
  'dark',
  'cupcake',
  'bumblebee',
  'emerald',
  'corporate',
  'synthwave',
  'retro',
  'cyberpunk',
  'valentine',
  'halloween',
  'garden',
  'forest',
  'aqua',
  'lofi',
  'pastel',
  'fantasy',
  'wireframe',
  'black',
  'luxury',
  'dracula',
  'cmyk',
  'autumn',
  'business',
  'acid',
  'lemonade',
  'night',
  'coffee',
  'winter',
  'dim',
  'nord',
  'sunset',
] as const;

export type DaisyUITheme = typeof AVAILABLE_THEMES[number];

// Theme store
const [currentTheme, setCurrentTheme] = createSignal<DaisyUITheme>('dark');
const [systemTheme, setSystemTheme] = createSignal<'light' | 'dark'>('dark');

// Initialize theme from localStorage
const initializeTheme = () => {
  const savedTheme = localStorage.getItem('app-theme') as DaisyUITheme;
  if (savedTheme && AVAILABLE_THEMES.includes(savedTheme)) {
    setCurrentTheme(savedTheme);
  }
  
  // Detect system theme
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  setSystemTheme(mediaQuery.matches ? 'dark' : 'light');
  
  // Listen for system theme changes
  mediaQuery.addEventListener('change', (e) => {
    setSystemTheme(e.matches ? 'dark' : 'light');
  });
};

// Apply theme to document
const applyTheme = (theme: DaisyUITheme) => {
  document.documentElement.setAttribute('data-theme', theme);
};

// Save theme to localStorage
createEffect(() => {
  const theme = currentTheme();
  localStorage.setItem('app-theme', theme);
  applyTheme(theme);
});

// Initialize on first load
initializeTheme();
applyTheme(currentTheme());

export const themeStore = {
  currentTheme,
  setCurrentTheme,
  systemTheme,
  availableThemes: AVAILABLE_THEMES,
};
