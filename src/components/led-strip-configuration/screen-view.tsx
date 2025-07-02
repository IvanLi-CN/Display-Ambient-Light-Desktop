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

  // Fetch screenshot data from backend
  const fetchScreenshot = async () => {
    console.log('📸 FETCH: Starting screenshot fetch', {
      isLoading: isLoading(),
      isMounted,
      hidden: hidden(),
      timestamp: new Date().toLocaleTimeString()
    });

    if (isLoading()) {
      console.log('⏳ FETCH: Already loading, skipping');
      return; // Skip if already loading
    }

    try {
      setIsLoading(true);

      const timestamp = Date.now();
      const response = await fetch(`ambient-light://displays/${localProps.displayId}?width=400&height=225&t=${timestamp}`);

      if (!response.ok) {
        console.error('❌ FETCH: Screenshot fetch failed', response.status, response.statusText);
        const errorText = await response.text();
        console.error('❌ FETCH: Error response body:', errorText);
        return;
      }

      const width = parseInt(response.headers.get('X-Image-Width') || '400');
      const height = parseInt(response.headers.get('X-Image-Height') || '225');
      const arrayBuffer = await response.arrayBuffer();
      const buffer = new Uint8ClampedArray(arrayBuffer);
      const expectedSize = width * height * 4;



      // Validate buffer size
      if (buffer.length !== expectedSize) {
        console.error('❌ FETCH: Invalid buffer size!', {
          received: buffer.length,
          expected: expectedSize,
          ratio: buffer.length / expectedSize
        });
        return;
      }

      console.log('📊 FETCH: Setting image data', { width, height, bufferSize: buffer.length });

      setImageData({
        buffer,
        width,
        height
      });

      // Use setTimeout to ensure the signal update has been processed
      setTimeout(() => {
        console.log('🖼️ FETCH: Triggering draw after data set');
        draw(false);
      }, 0);

      // Schedule next frame after rendering is complete
      const shouldContinue = !hidden() && isMounted;
      console.log('🔄 FETCH: Scheduling next frame', {
        hidden: hidden(),
        isMounted,
        shouldContinue,
        nextFrameDelay: '1000ms'
      });

      if (shouldContinue) {
        setTimeout(() => {
          if (isMounted) {
            console.log('🔄 FETCH: Starting next frame');
            fetchScreenshot();
          } else {
            console.log('❌ FETCH: Component unmounted, stopping loop');
          }
        }, 1000); // Wait 1 second before next frame
      } else {
        console.log('❌ FETCH: Loop stopped - component hidden or unmounted');
      }

    } catch (error) {
      console.error('❌ FETCH: Error fetching screenshot:', error);
      // Even on error, schedule next frame
      const shouldContinueOnError = !hidden() && isMounted;
      console.log('🔄 FETCH: Error recovery - scheduling next frame', {
        error: error.message,
        shouldContinue: shouldContinueOnError,
        nextFrameDelay: '2000ms'
      });

      if (shouldContinueOnError) {
        setTimeout(() => {
          if (isMounted) {
            console.log('🔄 FETCH: Retrying after error');
            fetchScreenshot();
          }
        }, 2000); // Wait longer on error
      }
    } finally {
      setIsLoading(false);
    }
  };

  const resetSize = () => {
    console.log('📏 CANVAS: Resizing', {
      rootClientWidth: root.clientWidth,
      rootClientHeight: root.clientHeight,
      oldCanvasWidth: canvas.width,
      oldCanvasHeight: canvas.height
    });

    // Set canvas size first
    canvas.width = root.clientWidth;
    canvas.height = root.clientHeight;

    console.log('📏 CANVAS: Size set', {
      newCanvasWidth: canvas.width,
      newCanvasHeight: canvas.height
    });

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

    console.log('🖼️ DRAW: Called with', {
      cached,
      hasContext: !!_ctx,
      hasImageData: !!raw,
      imageDataSize: raw ? `${raw.width}x${raw.height}` : 'none',
      drawInfo: { drawX, drawY, drawWidth, drawHeight },
      canvasSize: `${canvas.width}x${canvas.height}`,
      contextType: _ctx ? 'valid' : 'null',
      rawBufferSize: raw ? raw.buffer.length : 0
    });

    if (_ctx && raw) {
      console.log('🖼️ DRAW: Starting to draw image');
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
          console.log('🖼️ DRAW: Using putImageData directly');
          _ctx.putImageData(img, drawX, drawY);
          console.log('✅ DRAW: putImageData completed');
        } else {
          console.log('🖼️ DRAW: Using scaling with temp canvas');
          // Otherwise, use cached temporary canvas for scaling
          if (!tempCanvas || tempCanvas.width !== raw.width || tempCanvas.height !== raw.height) {
            tempCanvas = document.createElement('canvas');
            tempCanvas.width = raw.width;
            tempCanvas.height = raw.height;
            tempCtx = tempCanvas.getContext('2d');
            console.log('🖼️ DRAW: Created new temp canvas');
          }

          if (tempCtx) {
            tempCtx.putImageData(img, 0, 0);
            _ctx.drawImage(tempCanvas, drawX, drawY, drawWidth, drawHeight);
            console.log('✅ DRAW: Scaled drawing completed');
          }
        }
      } catch (error) {
        console.error('❌ DRAW: Error in draw():', error);
      }
    } else {
      console.log('❌ DRAW: Cannot draw - missing context or image data', {
        hasContext: !!_ctx,
        hasImageData: !!raw
      });
    }
  };



  // Initialize canvas and resize observer
  onMount(() => {
    console.log('🚀 CANVAS: Component mounted');
    const context = canvas.getContext('2d');
    console.log('🚀 CANVAS: Context obtained', !!context);
    setCtx(context);
    console.log('🚀 CANVAS: Context signal set');

    // Initial size setup
    resetSize();

    const resizeObserver = new ResizeObserver(() => {
      resetSize();
    });
    resizeObserver.observe(root);

    // Start screenshot fetching after context is ready
    console.log('🚀 SCREENSHOT: Starting screenshot fetching');
    setTimeout(() => {
      console.log('🚀 SCREENSHOT: Context ready, starting fetch');
      fetchScreenshot(); // Initial fetch - will self-schedule subsequent frames
    }, 100); // Small delay to ensure context is ready

    onCleanup(() => {
      isMounted = false; // Stop scheduling new frames
      resizeObserver?.unobserve(root);
      console.log('🧹 CLEANUP: Component unmounted');
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
