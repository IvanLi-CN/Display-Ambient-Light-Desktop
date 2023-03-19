import { Component } from 'solid-js';
import { DisplayInfo } from '../models/display-info.model';
import { DisplayInfoPanel } from './display-info-panel';
import { ScreenView } from './screen-view';

type DisplayViewProps = {
  display: DisplayInfo;
};

export const DisplayView: Component<DisplayViewProps> = (props) => {
  return (
    <section class="relative">
      <ScreenView displayId={props.display.id} />
      <DisplayInfoPanel
        display={props.display}
        class="absolute bg-slate-50/10 top-1/4 left-1/4 rounded backdrop-blur w-1/3 min-w-fit text-black"
      />
    </section>
  );
};
