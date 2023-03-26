import { DisplayConfig } from './display-config';

export class ScreenshotDto {
  encode_image!: string;
  config!: DisplayConfig;
  colors!: {
    top: Uint8Array;
    bottom: Uint8Array;
    left: Uint8Array;
    right: Uint8Array;
  };
}
