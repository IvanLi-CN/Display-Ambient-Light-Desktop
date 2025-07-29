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
  ) {}
}
