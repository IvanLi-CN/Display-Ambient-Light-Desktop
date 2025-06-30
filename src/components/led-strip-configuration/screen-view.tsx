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
  const [hidden, setHidden] = createSignal(false);

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

      // Skip if dimensions are not ready
      if (drawWidth <= 0 || drawHeight <= 0) {
        console.log('Skipping frame: invalid dimensions', { drawWidth, drawHeight });
        return;
      }

      const url = `ambient-light://displays/${localProps.displayId}?width=${drawWidth}&height=${drawHeight}`;

      console.log('Fetching screenshot:', url);

      try {
        const response = await fetch(url, {
          mode: 'cors',
        });

        if (!response.ok) {
          console.error('Screenshot fetch failed:', response.status, response.statusText);
          return;
        }

        const buffer = await response.body?.getReader().read();
        if (buffer?.value) {
          console.log('Screenshot received, size:', buffer.value.length);
          setImageData({
            buffer: new Uint8ClampedArray(buffer?.value),
            width: drawWidth,
            height: drawHeight,
          });
        } else {
          console.log('No screenshot data received');
          setImageData(null);
        }
        draw();
      } catch (error) {
        console.error('Screenshot fetch error:', error);
      }
    };

    (async () => {
      while (!stopped) {
        if (hidden()) {
          await new Promise((resolve) => setTimeout(resolve, 1000));
          continue;
        }

        await frame();

        // Add a small delay to prevent overwhelming the backend
        await new Promise((resolve) => setTimeout(resolve, 33)); // ~30 FPS
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

      // Initial size setup
      resetSize();

      resizeObserver = new ResizeObserver(() => {
        resetSize();
      });
      resizeObserver.observe(root);
    });

    onCleanup(() => {
      resizeObserver?.unobserve(root);
    });
  });

  // update hidden
  createEffect(() => {
    const hide = () => {
      setHidden(true);
      console.log('hide');
    };
    const show = () => {
      setHidden(false);
      console.log('show');
    };

    window.addEventListener('focus', show);
    window.addEventListener('blur', hide);

    onCleanup(() => {
      window.removeEventListener('focus', show);
      window.removeEventListener('blur', hide);
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
