import { createStore } from 'solid-js/store';
import {
  ColorCalibration,
  LedStripConfig,
  LedStripPixelMapper,
} from '../models/led-strip-config';

export const [ledStripStore, setLedStripStore] = createStore({
  strips: new Array<LedStripConfig>(),
  mappers: new Array<LedStripPixelMapper>(),
  colorCalibration: new ColorCalibration(),
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
