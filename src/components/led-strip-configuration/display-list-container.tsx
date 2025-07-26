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

    // 安全检查：确保边界值有效
    const boundWidth = _bound.right - _bound.left;
    const boundHeight = _bound.bottom - _bound.top;

    if (boundWidth <= 0 || boundHeight <= 0 || !isFinite(boundWidth) || !isFinite(boundHeight)) {
      console.warn('Invalid bounds detected, skipping resize:', _bound);
      return;
    }

    // Calculate and update view scale with persistence
    const newViewScale = root.clientWidth / boundWidth;

    // 额外的安全检查
    if (!isFinite(newViewScale) || newViewScale <= 0) {
      console.warn('Invalid view scale calculated, skipping update:', newViewScale);
      return;
    }

    await updateViewScale(newViewScale);

    setOlStyle({
      top: `${-_bound.top * displayStore.viewScale}px`,
      left: `${-_bound.left * displayStore.viewScale}px`,
    });

    setRootStyle({
      height: `${boundHeight * displayStore.viewScale}px`,
      background: `url(${background})`,
    });
  };

  createEffect(() => {
    // 安全检查：确保 displays 数组不为空
    if (!displayStore.displays || displayStore.displays.length === 0) {
      console.log('No displays available, using default bounds');
      setBound({
        left: 0,
        top: 0,
        right: 1920, // 默认宽度
        bottom: 1080, // 默认高度
      });
      return;
    }

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
