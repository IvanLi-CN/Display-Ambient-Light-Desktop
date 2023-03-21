import { Component, createMemo } from 'solid-js';
import { DisplayInfo } from '../models/display-info.model';
import { displayStore } from '../stores/display.store';
import { DisplayInfoPanel } from './display-info-panel';
import { ScreenView } from './screen-view';

type DisplayViewProps = {
  display: DisplayInfo;
};

export const DisplayView: Component<DisplayViewProps> = (props) => {
  const size = createMemo(() => ({
    width: props.display.width * displayStore.viewScale,
    height: props.display.height * displayStore.viewScale,
  }));
  const style = createMemo(() => ({
    top: `${props.display.y * displayStore.viewScale}px`,
    left: `${props.display.x * displayStore.viewScale}px`,
    height: `${size().height}px`,
    width: `${size().width}px`,
  }));
  return (
    <section
      class="absolute bg-gray-300 grid grid-cols-[16px,auto,16px] grid-rows-[16px,auto,16px] overflow-hidden"
      style={style()}
    >
      <ScreenView
        class="row-start-2 col-start-2"
        displayId={props.display.id}
        style={{
          'object-fit': 'contain',
        }}
      />
      <DisplayInfoPanel
        display={props.display}
        class="absolute bg-slate-50/10 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded backdrop-blur w-1/3 min-w-[300px] text-black"
      />
      <div class="row-start-1 col-start-2">Test</div>
      <div class="row-start-2 col-start-1">Test</div>
      <div class="row-start-2 col-start-3">Test</div>
      <div class="row-start-3 col-start-2">Test</div>
    </section>
  );
};
