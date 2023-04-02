import { createStore } from 'solid-js/store';
import { LedStripConfig, LedStripPixelMapper } from '../models/led-strip-config';

export const [ledStripStore, setLedStripStore] = createStore({
  strips: new Array<LedStripConfig>(),
  mappers: new Array<LedStripPixelMapper>(),
  colors: new Array<string>(),
  sortedColors: new Uint8ClampedArray(),
  get totalLedCount() {
    return Math.max(0, ...ledStripStore.mappers.map((m) => Math.max(m.start, m.end)));
  },
});
