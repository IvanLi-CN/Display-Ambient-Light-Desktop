export type DisplayState = {
  brightness: number;
  max_brightness: number;
  min_brightness: number;
  contrast: number;
  max_contrast: number;
  min_contrast: number;
  mode: number;
  max_mode: number;
  min_mode: number;
  last_modified_at: Date;
};

export type RawDisplayState = DisplayState & {
  last_modified_at: { secs_since_epoch: number };
};
