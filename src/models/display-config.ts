import { Borders } from '../constants/border';
import { LedStripConfig } from './led-strip-config';

export class LedStripConfigOfBorders implements Record<Borders, LedStripConfig | null> {
  constructor(
    public top: LedStripConfig | null = null,
    public bottom: LedStripConfig | null = null,
    public left: LedStripConfig | null = null,
    public right: LedStripConfig | null = null,
  ) {}
}
export class DisplayConfig {
  led_strip_of_borders = new LedStripConfigOfBorders();

  constructor(public id: number) {}
}
