import { createStore } from 'solid-js/store';
import { DisplayInfo } from '../models/display-info.model';
import { userPreferencesStore } from './user-preferences.store';

export const [displayStore, setDisplayStore] = createStore({
  displays: new Array<DisplayInfo>(),
  get viewScale() {
    return userPreferencesStore.viewScale;
  },
});

// Helper function to update view scale with persistence
export const updateViewScale = async (scale: number) => {
  try {
    await userPreferencesStore.updateViewScale(scale);
  } catch (error) {
    console.error('Failed to update view scale:', error);
  }
};
