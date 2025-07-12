import {
  createEffect,
  createSignal,
  JSX,
  onCleanup,
  onMount,
  ParentComponent,
} from 'solid-js';
import { displayStore, updateViewScale } from '../../stores/display.store';
import background from '../../assets/transparent-grid-background.svg?url';

export const DisplayListContainer: ParentComponent = (props) => {
  let root: HTMLElement;
  const [olStyle, setOlStyle] = createSignal({
    top: '0px',
    left: '0px',
  });
  const [rootStyle, setRootStyle] = createSignal<JSX.CSSProperties>({
    height: '100%',
  });
  const [bound, setBound] = createSignal({
    left: 0,
    top: 0,
    right: 100,
    bottom: 100,
  });

  const resetSize = async () => {
    const _bound = bound();

    // Calculate and update view scale with persistence
    const newViewScale = root.clientWidth / (_bound.right - _bound.left);
    await updateViewScale(newViewScale);

    setOlStyle({
      top: `${-_bound.top * displayStore.viewScale}px`,
      left: `${-_bound.left * displayStore.viewScale}px`,
    });

    setRootStyle({
      height: `${(_bound.bottom - _bound.top) * displayStore.viewScale}px`,
      background: `url(${background})`,
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
      observer = new ResizeObserver(() => {
        resetSize().catch(console.error);
      });
      observer.observe(root);
    });

    onCleanup(() => {
      observer?.unobserve(root);
    });
  });

  createEffect(() => {});

  return (
    <section ref={root!} class="relative bg-gray-400/30 h-full w-full" style={rootStyle()}>
      <ol class="absolute" style={olStyle()}>
        {props.children}
      </ol>
    </section>
  );
};
