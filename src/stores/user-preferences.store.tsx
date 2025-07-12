import { createSignal, createEffect, onMount } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

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
  },
};

// Reactive signals for user preferences
const [userPreferences, setUserPreferences] = createSignal<UserPreferences>(defaultPreferences);
const [isLoading, setIsLoading] = createSignal(false);

// Load preferences from backend
const loadPreferences = async () => {
  try {
    setIsLoading(true);
    const preferences = await invoke<UserPreferences>('get_user_preferences');
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
    await invoke('update_user_preferences', { preferences });
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
    await invoke('update_window_preferences', { windowPrefs });
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
    await invoke('update_ui_preferences', { uiPrefs });
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
    await invoke('update_view_scale', { scale });
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
    await invoke('update_theme', { theme });
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
    return await invoke<string>('get_theme');
  } catch (error) {
    console.error('Failed to get theme:', error);
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
  initializePreferences,

  // Getters
  get viewScale() {
    return userPreferences().ui.view_scale;
  },
  get theme() {
    return userPreferences().ui.theme;
  },
};
