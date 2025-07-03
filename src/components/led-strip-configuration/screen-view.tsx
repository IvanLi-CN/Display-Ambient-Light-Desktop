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

  // Cache temporary canvas for scaling
  let tempCanvas: HTMLCanvasElement | null = null;
  let tempCtx: CanvasRenderingContext2D | null = null;
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
  const [isLoading, setIsLoading] = createSignal(false);
  let isMounted = true;

  // Fetch screenshot data from backend with frame-based rendering
  const fetchScreenshot = async () => {
    if (isLoading()) {
      return; // Skip if already loading - frame-based approach
    }

    try {
      setIsLoading(true);

      const timestamp = Date.now();
      const response = await fetch(`ambient-light://displays/${localProps.displayId}?width=400&height=225&t=${timestamp}`);

      if (!response.ok) {
        console.error('Screenshot fetch failed:', response.status);
        return;
      }

      const width = parseInt(response.headers.get('X-Image-Width') || '400');
      const height = parseInt(response.headers.get('X-Image-Height') || '225');
      const arrayBuffer = await response.arrayBuffer();
      const buffer = new Uint8ClampedArray(arrayBuffer);
      const expectedSize = width * height * 4;

      // Validate buffer size
      if (buffer.length !== expectedSize) {
        console.error('Invalid buffer size:', buffer.length, 'expected:', expectedSize);
        return;
      }

      setImageData({
        buffer,
        width,
        height
      });

      // Draw immediately after data is set
      setTimeout(() => {
        draw(false);
      }, 0);

      // Frame-based rendering: wait for current frame to complete before scheduling next
      const shouldContinue = !hidden() && isMounted;
      if (shouldContinue) {
        setTimeout(() => {
          if (isMounted) {
            fetchScreenshot(); // Start next frame only after current one completes
          }
        }, 500); // Reduced frequency to 500ms for better performance
      }

    } catch (error) {
      console.error('Error fetching screenshot:', error);
      // On error, wait longer before retry
      const shouldContinueOnError = !hidden() && isMounted;
      if (shouldContinueOnError) {
        setTimeout(() => {
          if (isMounted) {
            fetchScreenshot();
          }
        }, 2000);
      }
    } finally {
      setIsLoading(false);
    }
  };

  const resetSize = () => {
    // Set canvas size first
    canvas.width = root.clientWidth;
    canvas.height = root.clientHeight;

    // Use a default aspect ratio if canvas dimensions are invalid
    const aspectRatio = (canvas.width > 0 && canvas.height > 0)
      ? canvas.width / canvas.height
      : 16 / 9; // Default 16:9 aspect ratio

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

    draw(true);
  };

  const draw = (cached: boolean = false) => {
    const { drawX, drawY, drawWidth, drawHeight } = drawInfo();
    let _ctx = ctx();
    let raw = imageData();

    if (_ctx && raw) {
      _ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Apply transparency effect for cached images if needed
      let buffer = raw.buffer;
      if (cached) {
        buffer = new Uint8ClampedArray(raw.buffer);
        for (let i = 3; i < buffer.length; i += 4) {
          buffer[i] = Math.floor(buffer[i] * 0.7);
        }
      }

      try {
        // Create ImageData and draw directly
        const img = new ImageData(buffer, raw.width, raw.height);

        // If the image size matches the draw size, use putImageData directly
        if (raw.width === drawWidth && raw.height === drawHeight) {
          _ctx.putImageData(img, drawX, drawY);
        } else {
          // Otherwise, use cached temporary canvas for scaling
          if (!tempCanvas || tempCanvas.width !== raw.width || tempCanvas.height !== raw.height) {
            tempCanvas = document.createElement('canvas');
            tempCanvas.width = raw.width;
            tempCanvas.height = raw.height;
            tempCtx = tempCanvas.getContext('2d');
          }

          if (tempCtx) {
            tempCtx.putImageData(img, 0, 0);
            _ctx.drawImage(tempCanvas, drawX, drawY, drawWidth, drawHeight);
          }
        }
      } catch (error) {
        console.error('Error in draw():', error);
      }
    }
  };



  // Initialize canvas and resize observer
  onMount(() => {
    const context = canvas.getContext('2d');
    setCtx(context);

    // Initial size setup
    resetSize();

    const resizeObserver = new ResizeObserver(() => {
      resetSize();
    });
    resizeObserver.observe(root);

    // Start screenshot fetching after context is ready
    setTimeout(() => {
      fetchScreenshot(); // Initial fetch - will self-schedule subsequent frames
    }, 100); // Small delay to ensure context is ready

    onCleanup(() => {
      isMounted = false; // Stop scheduling new frames
      resizeObserver?.unobserve(root);
    });
  });



  // Note: Removed window focus/blur logic as it was causing screenshot loop to stop
  // when user interacted with dev tools or other windows

  return (
    <div
      ref={root!}
      {...rootProps}
      class={'overflow-hidden h-full w-full ' + rootProps.class}
    >
      <canvas
        ref={canvas!}
        style={{
          display: 'block',
          width: '100%',
          height: '100%',
          'background-color': '#f0f0f0'
        }}
      />
      {rootProps.children}
    </div>
  );
};
