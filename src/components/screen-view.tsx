import { invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import {
  Component,
  createEffect,
  createSignal,
  JSX,
  onCleanup,
  onMount,
  splitProps,
} from 'solid-js';

type ScreenViewProps = {
  displayId: number;
} & JSX.HTMLAttributes<HTMLDivElement>;

export const ScreenView: Component<ScreenViewProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['displayId']);
  let canvas: HTMLCanvasElement;
  let root: HTMLDivElement;
  const [ctx, setCtx] = createSignal<CanvasRenderingContext2D | null>(null);
  const [drawInfo, setDrawInfo] = createSignal({
    drawX: 0,
    drawY: 0,
    drawWidth: 0,
    drawHeight: 0,
  });
  const [imageData, setImageData] = createSignal<{
    buffer: Uint8ClampedArray;
    width: number;
    height: number;
  } | null>(null);

  const resetSize = () => {
    const aspectRatio = canvas.width / canvas.height;

    const drawWidth = Math.round(
      Math.min(root.clientWidth, root.clientHeight * aspectRatio),
    );
    const drawHeight = Math.round(
      Math.min(root.clientHeight, root.clientWidth / aspectRatio),
    );

    const drawX = Math.round((root.clientWidth - drawWidth) / 2);
    const drawY = Math.round((root.clientHeight - drawHeight) / 2);

    setDrawInfo({
      drawX,
      drawY,
      drawWidth,
      drawHeight,
    });

    canvas.width = root.clientWidth;
    canvas.height = root.clientHeight;

    draw(true);
  };

  const draw = (cached: boolean = false) => {
    const { drawX, drawY } = drawInfo();

    let _ctx = ctx();
    let raw = imageData();
    if (_ctx && raw) {
      _ctx.clearRect(0, 0, canvas.width, canvas.height);
      if (cached) {
        for (let i = 3; i < raw.buffer.length; i += 8) {
          raw.buffer[i] = Math.floor(raw.buffer[i] * 0.7);
        }
      }
      const img = new ImageData(raw.buffer, raw.width, raw.height);
      _ctx.putImageData(img, drawX, drawY);
    }
  };

  // get screenshot
  createEffect(() => {
    let stopped = false;
    const frame = async () => {
      const { drawWidth, drawHeight } = drawInfo();
      const url = convertFileSrc(
        `displays/${localProps.displayId}?width=${drawWidth}&height=${drawHeight}`,
        'ambient-light',
      );
      await fetch(url, {
        mode: 'cors',
      })
        .then((res) => res.body?.getReader().read())
        .then((buffer) => {
          if (buffer?.value) {
            setImageData({
              buffer: new Uint8ClampedArray(buffer?.value),
              width: drawWidth,
              height: drawHeight,
            });
          } else {
            setImageData(null);
          }
          draw();
        });
    };

    (async () => {
      while (!stopped) {
        await frame();
      }
    })();

    onCleanup(() => {
      stopped = true;
    });
  });

  // resize
  createEffect(() => {
    let resizeObserver: ResizeObserver;

    onMount(() => {
      setCtx(canvas.getContext('2d'));
      new ResizeObserver(() => {
        resetSize();
      }).observe(root);
    });

    onCleanup(() => {
      resizeObserver?.unobserve(root);
    });
  });

  return (
    <div
      ref={root!}
      {...rootProps}
      class={'overflow-hidden h-full w-full ' + rootProps.class}
    >
      <canvas ref={canvas!} />
      {rootProps.children}
    </div>
  );
};
