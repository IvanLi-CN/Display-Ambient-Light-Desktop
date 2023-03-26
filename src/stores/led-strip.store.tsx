import { createStore } from 'solid-js/store';
import { DisplayConfig } from '../models/display-config';
import { LedStripConfig } from '../models/led-strip-config';

export const [ledStripStore, setLedStripStore] = createStore({
  displays: new Array<DisplayConfig>(),
  strips: new Array<LedStripConfig>(),
});
