import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
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
  height: number;
  width: number;
} & Omit<JSX.HTMLAttributes<HTMLCanvasElement>, 'height' | 'width'>;

async function subscribeScreenshotUpdate(displayId: number) {
  await invoke('subscribe_encoded_screenshot_updated', {
    displayId,
  });
}

export const ScreenView: Component<ScreenViewProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['displayId']);
  let canvas: HTMLCanvasElement;
  const [ctx, setCtx] = createSignal<CanvasRenderingContext2D | null>(null);
  createEffect(() => {
    const unlisten = listen<{
      base64_image: string;
      display_id: number;
      height: number;
      width: number;
    }>('encoded-screenshot-updated', (event) => {
      if (event.payload.display_id === localProps.displayId) {
        const url = convertFileSrc(
          `displays/${localProps.displayId}?width=${canvas.width}&height=${canvas.height}`,
          'ambient-light',
        );
        fetch(url, {
          mode: 'cors',
        })
          .then((res) => res.body?.getReader().read())
          .then((buffer) => {
            console.log(buffer?.value?.length);

            let _ctx = ctx();
            if (_ctx && buffer?.value) {
              _ctx.clearRect(0, 0, canvas.width, canvas.height);
              const img = new ImageData(
                new Uint8ClampedArray(buffer.value),
                canvas.width,
                canvas.height,
              );
              _ctx.putImageData(img, 0, 0);
            }
          });
      }

      // console.log(event.payload.display_id, localProps.displayId);
    });
    subscribeScreenshotUpdate(localProps.displayId);

    onMount(() => {
      setCtx(canvas.getContext('2d'));
    });

    onCleanup(() => {
      unlisten.then((unlisten) => {
        unlisten();
      });
    });
  });

  return <canvas ref={canvas!} class="object-contain" {...rootProps} />;
};
