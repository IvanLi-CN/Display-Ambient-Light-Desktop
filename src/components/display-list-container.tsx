import {
  createEffect,
  createSignal,
  onCleanup,
  onMount,
  ParentComponent,
} from 'solid-js';
import { displayStore, setDisplayStore } from '../stores/display.store';

export const DisplayListContainer: ParentComponent = (props) => {
  let root: HTMLElement;
  const [olStyle, setOlStyle] = createSignal({
    top: '0px',
    left: '0px',
  });
  const [rootStyle, setRootStyle] = createSignal({
    // width: '100%',
    height: '100%',
  });
  const [bound, setBound] = createSignal({
    left: 0,
    top: 0,
    right: 100,
    bottom: 100,
  });

  const resetSize = () => {
    const _bound = bound();

    setDisplayStore({
      viewScale: root.clientWidth / (_bound.right - _bound.left),
    });

    setOlStyle({
      top: `${-_bound.top * displayStore.viewScale}px`,
      left: `${-_bound.left * displayStore.viewScale}px`,
    });

    setRootStyle({
      height: `${(_bound.bottom - _bound.top) * displayStore.viewScale}px`,
    });
  };

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

    setBound({
      left: boundLeft,
      top: boundTop,
      right: boundRight,
      bottom: boundBottom,
    });
    let observer: ResizeObserver;
    onMount(() => {
      observer = new ResizeObserver(resetSize);
      observer.observe(root);
    });

    onCleanup(() => {
      observer?.unobserve(root);
    });
  });

  createEffect(() => {});

  return (
    <section ref={root!} class="relative bg-gray-400/30" style={rootStyle()}>
      <ol class="absolute bg-gray-700" style={olStyle()}>
        {props.children}
      </ol>
    </section>
  );
};
