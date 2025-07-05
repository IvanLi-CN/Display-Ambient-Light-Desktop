import {
  Component,
  createEffect,
  createSignal,
  JSX,
  onCleanup,
  onMount,
  splitProps,
} from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

type ScreenViewWebSocketProps = {
  displayId: number;
  width?: number;
  height?: number;
  quality?: number;
} & JSX.HTMLAttributes<HTMLDivElement>;

export const ScreenViewWebSocket: Component<ScreenViewWebSocketProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['displayId', 'width', 'height', 'quality']);
  let canvas: HTMLCanvasElement;
  let root: HTMLDivElement;
  const [ctx, setCtx] = createSignal<CanvasRenderingContext2D | null>(null);

  const [drawInfo, setDrawInfo] = createSignal({
    drawX: 0,
    drawY: 0,
    drawWidth: 0,
    drawHeight: 0,
  });

  const [connectionStatus, setConnectionStatus] = createSignal<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const [frameCount, setFrameCount] = createSignal(0);
  const [lastFrameTime, setLastFrameTime] = createSignal(0);
  const [fps, setFps] = createSignal(0);

  let websocket: WebSocket | null = null;
  let reconnectTimeout: number | null = null;
  let isMounted = true;

  // Performance monitoring
  let frameTimestamps: number[] = [];

  const connectWebSocket = () => {
    if (!isMounted) {
      return;
    }

    const wsUrl = `ws://127.0.0.1:8765`;

    setConnectionStatus('connecting');
    websocket = new WebSocket(wsUrl);
    websocket.binaryType = 'arraybuffer';

    websocket.onopen = () => {
      setConnectionStatus('connected');

      // Send initial configuration
      const config = {
        display_id: localProps.displayId,
        width: localProps.width || 320,
        height: localProps.height || 180,
        quality: localProps.quality || 50
      };
      websocket?.send(JSON.stringify(config));
    };

    websocket.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        handleJpegFrame(new Uint8Array(event.data));
      }
    };

    websocket.onclose = (event) => {
      setConnectionStatus('disconnected');
      websocket = null;
      
      // Auto-reconnect after 2 seconds if component is still mounted
      if (isMounted && !reconnectTimeout) {
        reconnectTimeout = window.setTimeout(() => {
          reconnectTimeout = null;
          connectWebSocket();
        }, 2000);
      }
    };

    websocket.onerror = (error) => {
      setConnectionStatus('error');
    };
  };

  const handleJpegFrame = async (jpegData: Uint8Array) => {
    const _ctx = ctx();
    if (!_ctx) return;

    try {
      // Update performance metrics
      const now = performance.now();
      frameTimestamps.push(now);
      
      // Keep only last 30 frames for FPS calculation
      if (frameTimestamps.length > 30) {
        frameTimestamps = frameTimestamps.slice(-30);
      }
      
      // Calculate FPS
      if (frameTimestamps.length >= 2) {
        const timeSpan = frameTimestamps[frameTimestamps.length - 1] - frameTimestamps[0];
        if (timeSpan > 0) {
          const currentFps = Math.round((frameTimestamps.length - 1) * 1000 / timeSpan);
          setFps(Math.max(0, currentFps)); // Ensure FPS is never negative
        }
      }

      setFrameCount(prev => prev + 1);
      setLastFrameTime(now);

      // Create blob from JPEG data
      const blob = new Blob([jpegData], { type: 'image/jpeg' });
      const imageUrl = URL.createObjectURL(blob);

      // Create image element
      const img = new Image();
      img.onload = () => {
        const { drawX, drawY, drawWidth, drawHeight } = drawInfo();
        
        // Clear canvas
        _ctx.clearRect(0, 0, canvas.width, canvas.height);
        
        // Draw image
        _ctx.drawImage(img, drawX, drawY, drawWidth, drawHeight);
        
        // Clean up
        URL.revokeObjectURL(imageUrl);
      };
      
      img.onerror = () => {
        console.error('Failed to load JPEG image');
        URL.revokeObjectURL(imageUrl);
      };
      
      img.src = imageUrl;

    } catch (error) {
      console.error('Error handling JPEG frame:', error);
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
  };

  const disconnect = () => {
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }
    
    if (websocket) {
      websocket.close();
      websocket = null;
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

    // Connect WebSocket
    connectWebSocket();

    onCleanup(() => {
      isMounted = false;
      disconnect();
      resizeObserver?.unobserve(root);
    });
  });



  // Status indicator
  const getStatusColor = () => {
    switch (connectionStatus()) {
      case 'connected': return '#10b981'; // green
      case 'connecting': return '#f59e0b'; // yellow
      case 'error': return '#ef4444'; // red
      default: return '#6b7280'; // gray
    }
  };

  return (
    <div
      ref={root!}
      {...rootProps}
      class={'overflow-hidden h-full w-full relative ' + (rootProps.class || '')}
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
      
      {/* Status indicator */}
      <div class="absolute top-2 right-2 flex items-center gap-2 bg-black bg-opacity-50 text-white px-2 py-1 rounded text-xs">
        <div
          class="w-2 h-2 rounded-full"
          style={{ 'background-color': getStatusColor() }}
        />
        <span>{connectionStatus()}</span>
        {connectionStatus() === 'connected' && (
          <span>| {fps()} FPS | {frameCount()} frames</span>
        )}
      </div>
      
      {rootProps.children}
    </div>
  );
};
