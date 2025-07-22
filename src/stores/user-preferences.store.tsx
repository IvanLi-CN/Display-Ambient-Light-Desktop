import { createSignal } from 'solid-js';
import { adaptiveApi } from '../services/api-adapter';

// TypeScript interfaces matching Rust structs
export interface UserPreferences {
  window: WindowPreferences;
  ui: UIPreferences;
}

export interface WindowPreferences {
  width: number;
  height: number;
  x?: number;
  y?: number;
  maximized: boolean;
  minimized_to_tray: boolean;
}

export interface UIPreferences {
  view_scale: number;
  theme: string;
  night_mode_theme_enabled: boolean;
  night_mode_theme: string;
}

// Default preferences
const defaultPreferences: UserPreferences = {
  window: {
    width: 1400,
    height: 1000,
    maximized: false,
    minimized_to_tray: false,
  },
  ui: {
    view_scale: 0.2,
    theme: 'dark',
    night_mode_theme_enabled: false,
    night_mode_theme: 'dark',
  },
};

// Reactive signals for user preferences
const [userPreferences, setUserPreferences] = createSignal<UserPreferences>(defaultPreferences);
const [isLoading, setIsLoading] = createSignal(false);

// Load preferences from backend
const loadPreferences = async () => {
  try {
    setIsLoading(true);
    const preferences = await adaptiveApi.getUserPreferences();
    setUserPreferences(preferences);
    console.log('User preferences loaded:', preferences);
  } catch (error) {
    console.error('Failed to load user preferences:', error);
    // Use default preferences on error
    setUserPreferences(defaultPreferences);
  } finally {
    setIsLoading(false);
  }
};

// Save preferences to backend
const savePreferences = async (preferences: UserPreferences) => {
  try {
    await adaptiveApi.updateUserPreferences(preferences);
    setUserPreferences(preferences);
    console.log('User preferences saved:', preferences);
  } catch (error) {
    console.error('Failed to save user preferences:', error);
    throw error;
  }
};

// Update specific preference sections
const updateWindowPreferences = async (windowPrefs: WindowPreferences) => {
  try {
    await adaptiveApi.updateWindowPreferences(windowPrefs);
    setUserPreferences(prev => ({
      ...prev,
      window: windowPrefs,
    }));
  } catch (error) {
    console.error('Failed to update window preferences:', error);
    throw error;
  }
};

const updateUIPreferences = async (uiPrefs: UIPreferences) => {
  try {
    await adaptiveApi.updateUIPreferences(uiPrefs);
    setUserPreferences(prev => ({
      ...prev,
      ui: uiPrefs,
    }));
  } catch (error) {
    console.error('Failed to update UI preferences:', error);
    throw error;
  }
};

// Removed updateDisplayPreferences - feature not implemented

// Convenience functions for common updates
const updateViewScale = async (scale: number) => {
  try {
    await adaptiveApi.updateViewScale(scale);
    setUserPreferences(prev => ({
      ...prev,
      ui: {
        ...prev.ui,
        view_scale: scale,
      },
    }));
  } catch (error) {
    console.error('Failed to update view scale:', error);
    throw error;
  }
};

const updateTheme = async (theme: string) => {
  try {
    await adaptiveApi.updateTheme(theme);
    setUserPreferences(prev => ({
      ...prev,
      ui: {
        ...prev.ui,
        theme: theme,
      },
    }));
  } catch (error) {
    console.error('Failed to update theme:', error);
    throw error;
  }
};

const getTheme = async (): Promise<string> => {
  try {
    return await adaptiveApi.getTheme();
  } catch (error) {
    console.error('Failed to get theme:', error);
    return 'dark'; // fallback
  }
};

const updateNightModeThemeEnabled = async (enabled: boolean) => {
  try {
    await adaptiveApi.updateNightModeThemeEnabled(enabled);
    setUserPreferences(prev => ({
      ...prev,
      ui: {
        ...prev.ui,
        night_mode_theme_enabled: enabled,
      },
    }));
  } catch (error) {
    console.error('Failed to update night mode theme enabled:', error);
    throw error;
  }
};

const updateNightModeTheme = async (theme: string) => {
  try {
    await adaptiveApi.updateNightModeTheme(theme);
    setUserPreferences(prev => ({
      ...prev,
      ui: {
        ...prev.ui,
        night_mode_theme: theme,
      },
    }));
  } catch (error) {
    console.error('Failed to update night mode theme:', error);
    throw error;
  }
};

const getNightModeThemeEnabled = async (): Promise<boolean> => {
  try {
    return await adaptiveApi.getNightModeThemeEnabled();
  } catch (error) {
    console.error('Failed to get night mode theme enabled:', error);
    return false; // fallback
  }
};

const getNightModeTheme = async (): Promise<string> => {
  try {
    return await adaptiveApi.getNightModeTheme();
  } catch (error) {
    console.error('Failed to get night mode theme:', error);
    return 'dark'; // fallback
  }
};

// Removed updateLastVisitedPage - feature not implemented

// Removed helper functions for unimplemented features

// Initialize preferences on first load
let initialized = false;
const initializePreferences = async () => {
  if (!initialized) {
    initialized = true;
    await loadPreferences();
  }
};

// Export the store
export const userPreferencesStore = {
  // Signals
  preferences: userPreferences,
  isLoading,

  // Actions
  loadPreferences,
  savePreferences,
  updateWindowPreferences,
  updateUIPreferences,
  updateViewScale,
  updateTheme,
  getTheme,
  updateNightModeThemeEnabled,
  updateNightModeTheme,
  getNightModeThemeEnabled,
  getNightModeTheme,
  initializePreferences,

  // Getters
  get viewScale() {
    return userPreferences().ui.view_scale;
  },
  get theme() {
    return userPreferences().ui.theme;
  },
  get nightModeThemeEnabled() {
    return userPreferences().ui.night_mode_theme_enabled;
  },
  get nightModeTheme() {
    return userPreferences().ui.night_mode_theme;
  },
};
