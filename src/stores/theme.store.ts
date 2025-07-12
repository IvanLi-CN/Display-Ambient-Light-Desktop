import { createSignal, createEffect } from 'solid-js';
import { userPreferencesStore } from './user-preferences.store';

export type Theme = 'light' | 'dark' | 'auto';

// Available DaisyUI themes (DaisyUI 5.0)
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
  'caramellatte',
  'abyss',
  'silk',
] as const;

export type DaisyUITheme = typeof AVAILABLE_THEMES[number];

// Theme store
const [currentTheme, setCurrentTheme] = createSignal<DaisyUITheme>('dark');
const [systemTheme, setSystemTheme] = createSignal<'light' | 'dark'>('dark');
const [isInitialized, setIsInitialized] = createSignal(false);

// Initialize theme from backend
const initializeTheme = async () => {
  try {
    // Initialize user preferences first
    await userPreferencesStore.initializePreferences();

    // Get theme from backend
    const savedTheme = await userPreferencesStore.getTheme();
    if (savedTheme && AVAILABLE_THEMES.includes(savedTheme as DaisyUITheme)) {
      setCurrentTheme(savedTheme as DaisyUITheme);
    }

    // Detect system theme
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    setSystemTheme(mediaQuery.matches ? 'dark' : 'light');

    // Listen for system theme changes
    mediaQuery.addEventListener('change', (e) => {
      setSystemTheme(e.matches ? 'dark' : 'light');
    });

    // Mark as initialized
    setIsInitialized(true);
  } catch (error) {
    console.warn('Failed to initialize theme:', error);
    setIsInitialized(true); // Still mark as initialized to allow saving
  }
};

// Apply theme to document
const applyTheme = (theme: DaisyUITheme) => {
  try {
    document.documentElement.setAttribute('data-theme', theme);
  } catch (error) {
    console.warn('Failed to apply theme:', error);
  }
};

// Save theme to backend (only after initialization)
createEffect(() => {
  const theme = currentTheme();
  const initialized = isInitialized();

  // Always apply theme to UI
  applyTheme(theme);

  // Only save to backend after initialization to avoid overwriting loaded theme
  if (initialized) {
    userPreferencesStore.updateTheme(theme).catch(error => {
      console.error('Failed to save theme to backend:', error);
    });
  }
});

// Initialize on first load
if (typeof window !== 'undefined') {
  initializeTheme().then(() => {
    applyTheme(currentTheme());
  }).catch(error => {
    console.error('Failed to initialize theme:', error);
    applyTheme(currentTheme()); // Apply default theme
  });
}

export const themeStore = {
  currentTheme,
  setCurrentTheme,
  systemTheme,
  availableThemes: AVAILABLE_THEMES,
};
