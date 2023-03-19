import { Component, createMemo } from 'solid-js';
import { DisplayInfo } from '../models/display-info.model';
import { displayStore } from '../stores/display.store';
import { DisplayInfoPanel } from './display-info-panel';
import { ScreenView } from './screen-view';

type DisplayViewProps = {
  display: DisplayInfo;
};

export const DisplayView: Component<DisplayViewProps> = (props) => {
  const style = createMemo(() => ({
    width: `${props.display.width * displayStore.viewScale}px`,
    height: `${props.display.height * displayStore.viewScale}px`,
    top: `${props.display.y * displayStore.viewScale}px`,
    left: `${props.display.x * displayStore.viewScale}px`,
  }));
  return (
    <section class="absolute bg-gray-300" style={style()}>
      <ScreenView displayId={props.display.id} />
      <DisplayInfoPanel
        display={props.display}
        class="absolute bg-slate-50/10 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded backdrop-blur w-1/3 min-w-[300px] text-black"
      />
    </section>
  );
};
