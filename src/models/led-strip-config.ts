import { Borders } from '../constants/border';

export type LedStripPixelMapper = {
  start: number;
  end: number;
  pos: number;
};

export type LedStripConfigContainer = {
  strips: LedStripConfig[];
  mappers: LedStripPixelMapper[];
};

export class LedStripConfig {
  constructor(
    public readonly display_id: number,
    public readonly border: Borders,
    public len: number,
  ) {}
}
