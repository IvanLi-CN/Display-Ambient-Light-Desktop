import { Borders } from '../constants/border';

export class LedStripConfig {
  constructor(
    public readonly display_id: number,
    public readonly border: Borders,
    public start_pos: number,
    public len: number,
  ) {}
}
