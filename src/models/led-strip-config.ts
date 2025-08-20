import { Borders } from '../constants/border';

export enum LedType {
  WS2812B = 'WS2812B',
  SK6812 = 'SK6812',
}



export class ColorCalibration {
  r: number = 1;
  g: number = 1;
  b: number = 1;
  w: number = 1;
}

export type LedStripConfigContainer = {
  strips: LedStripConfig[];
  color_calibration: ColorCalibration;
};

export class LedStripConfig {
  constructor(
    public readonly index: number,
    public readonly display_id: number,
    public readonly border: Borders,
    public len: number,
    public led_type: LedType = LedType.WS2812B,
    public readonly reversed: boolean = false,
    /**
     * V2配置格式支持：显示器内部ID
     * 用于稳定的显示器标识，不受系统重启影响
     */
    public readonly display_internal_id?: string,
  ) {}

  /**
   * 获取用于匹配的显示器标识符
   * 优先使用内部ID，回退到数字ID
   */
  getDisplayIdentifier(): string | number {
    return this.display_internal_id || this.display_id;
  }

  /**
   * 检查是否匹配指定的显示器
   * 支持数字ID和内部ID两种匹配方式
   */
  matchesDisplay(displayId: number, displayInternalId?: string): boolean {
    // 如果有内部ID，优先使用内部ID匹配
    if (this.display_internal_id && displayInternalId) {
      return this.display_internal_id === displayInternalId;
    }
    // 回退到数字ID匹配
    return this.display_id === displayId;
  }
}
