import { createStore } from 'solid-js/store';
import { DisplayInfo } from '../models/display-info.model';

export const [displayStore, setDisplayStore] = createStore({
  displays: new Array<DisplayInfo>(),
  viewScale: 0.2,
});
