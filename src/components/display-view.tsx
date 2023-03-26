import { Component, createMemo } from 'solid-js';
import { LedStripConfigOfBorders } from '../models/display-config';
import { DisplayInfo } from '../models/display-info.model';
import { displayStore } from '../stores/display.store';
import { ledStripStore } from '../stores/led-strip.store';
import { DisplayInfoPanel } from './display-info-panel';
import { LedStripPart } from './led-strip-part';
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

  const ledStripConfigs = createMemo(() => {
    console.log('ledStripConfigs', ledStripStore.strips);
    return ledStripStore.strips.filter((c) => c.display_id === props.display.id);
  });

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
      <LedStripPart
        class="row-start-1 col-start-2 flex-row"
        config={ledStripConfigs().find((c) => c.border === 'Top')}
      />
      <LedStripPart
        class="row-start-2 col-start-1 flex-col"
        config={ledStripConfigs().find((c) => c.border === 'Left')}
      />
      <LedStripPart
        class="row-start-2 col-start-3 flex-col"
        config={ledStripConfigs().find((c) => c.border === 'Right')}
      />
      <LedStripPart
        class="row-start-3 col-start-2 flex-row"
        config={ledStripConfigs().find((c) => c.border === 'Bottom')}
      />
    </section>
  );
};
