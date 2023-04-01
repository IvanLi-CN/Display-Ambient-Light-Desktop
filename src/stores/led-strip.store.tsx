import { createStore } from 'solid-js/store';
import { DisplayConfig } from '../models/display-config';
import { LedStripConfig, LedStripPixelMapper } from '../models/led-strip-config';

export const [ledStripStore, setLedStripStore] = createStore({
  displays: new Array<DisplayConfig>(),
  strips: new Array<LedStripConfig>(),
  mappers: new Array<LedStripPixelMapper>(),
  colors: new Uint8ClampedArray(),
});
