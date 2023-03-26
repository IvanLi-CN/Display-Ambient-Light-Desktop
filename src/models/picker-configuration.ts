import { DisplayConfig } from './display-config';

export class PickerConfiguration {
  constructor(
    public display_configs: DisplayConfig[] = [],
    public config_version: number = 1,
  ) {}
}
