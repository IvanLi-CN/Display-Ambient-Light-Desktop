import { createEffect, createMemo, createSignal, on, ParentComponent } from 'solid-js';
import { displayStore, setDisplayStore } from '../stores/display.store';

export const DisplayListContainer: ParentComponent = (props) => {
  const [olStyle, setOlStyle] = createSignal({
    top: '0px',
    left: '0px',
  });
  const [rootStyle, setRootStyle] = createSignal({
    width: '100%',
    height: '100%',
  });

  createEffect(() => {
    const boundLeft = Math.min(0, ...displayStore.displays.map((display) => display.x));
    const boundTop = Math.min(0, ...displayStore.displays.map((display) => display.y));
    const boundRight = Math.max(
      0,
      ...displayStore.displays.map((display) => display.x + display.width),
    );
    const boundBottom = Math.max(
      0,
      ...displayStore.displays.map((display) => display.y + display.height),
    );

    setDisplayStore({
      viewScale: 1200 / (boundRight - boundLeft),
    });

    setOlStyle({
      top: `${-boundTop * displayStore.viewScale}px`,
      left: `${-boundLeft * displayStore.viewScale}px`,
    });

    setRootStyle({
      width: `${(boundRight - boundLeft) * displayStore.viewScale}px`,
      height: `${(boundBottom - boundTop) * displayStore.viewScale}px`,
    });
  });
  return (
    <section class="relative bg-gray-400/30" style={rootStyle()}>
      <ol class="absolute bg-gray-700" style={olStyle()}>
        {props.children}
      </ol>
    </section>
  );
};
