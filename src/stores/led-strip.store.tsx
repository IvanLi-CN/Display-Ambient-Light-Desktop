import { createStore } from 'solid-js/store';
import {
  ColorCalibration,
  LedStripConfig,
} from '../models/led-strip-config';

export const [ledStripStore, setLedStripStore] = createStore({
  strips: new Array<LedStripConfig>(),
  colorCalibration: new ColorCalibration(),
  colors: new Uint8ClampedArray(),
  sortedColors: new Uint8ClampedArray(),
  get totalLedCount() {
    // 计算总LED数量基于strips配置
    return ledStripStore.strips.reduce((total, strip) => {
      return total + strip.len;
    }, 0);
  },
});
