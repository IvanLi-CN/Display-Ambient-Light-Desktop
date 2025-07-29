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
  // 按灯带分组的颜色数据：key为 "display_id:border:strip_index"，value为颜色数组
  stripColors: new Map<string, Uint8ClampedArray>(),
  get totalLedCount() {
    // 计算总LED数量基于strips配置
    return ledStripStore.strips.reduce((total, strip) => {
      return total + strip.len;
    }, 0);
  },
  // 从分组的灯带颜色重新组装全局颜色数组（向后兼容）
  get assembledColors() {
    if (ledStripStore.stripColors.size === 0) {
      return ledStripStore.colors; // 回退到原始颜色数组
    }

    // 按灯带索引排序并组装颜色
    const sortedStrips = [...ledStripStore.strips].sort((a, b) => a.index - b.index);
    const assembledBytes: number[] = [];

    for (const strip of sortedStrips) {
      const stripKey = `${strip.display_id}:${strip.border}:${strip.index}`;
      const stripColors = ledStripStore.stripColors.get(stripKey);
      if (stripColors) {
        assembledBytes.push(...Array.from(stripColors));
      }
    }

    return new Uint8ClampedArray(assembledBytes);
  },
});
