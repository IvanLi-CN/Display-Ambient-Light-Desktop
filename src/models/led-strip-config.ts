import { Borders } from '../constants/border';

export enum LedType {
  WS2812B = 'WS2812B',
  SK6812 = 'SK6812',
}

export type LedStripPixelMapper = {
  start: number;
  end: number;
  pos: number;
};

export class ColorCalibration {
  r: number = 1;
  g: number = 1;
  b: number = 1;
  w: number = 1;
}

export type LedStripConfigContainer = {
  strips: LedStripConfig[];
  mappers: LedStripPixelMapper[];
  color_calibration: ColorCalibration;
};

export class LedStripConfig {
  constructor(
    public readonly display_id: number,
    public readonly border: Borders,
    public len: number,
    public led_type: LedType = LedType.WS2812B,
  ) {}
}
