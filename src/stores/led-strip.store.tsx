import { createStore } from 'solid-js/store';
import { LedStripConfig, LedStripPixelMapper } from '../models/led-strip-config';

export const [ledStripStore, setLedStripStore] = createStore({
  strips: new Array<LedStripConfig>(),
  mappers: new Array<LedStripPixelMapper>(),
  colors: new Uint8ClampedArray(),
  sortedColors: new Uint8ClampedArray(),
  get totalLedCount() {
    return Math.max(
      0,
      ...ledStripStore.mappers.map((m) => {
        if (m.start === m.end) {
          return 0;
        } else {
          return Math.max(m.start, m.end);
        }
      }),
    );
  },
});
