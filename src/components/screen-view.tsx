import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import {
  Component,
  createEffect,
  createSignal,
  JSX,
  onCleanup,
  splitProps,
} from 'solid-js';

type ScreenViewProps = {
  displayId: number;
} & JSX.HTMLAttributes<HTMLImageElement>;

async function subscribeScreenshotUpdate(displayId: number) {
  await invoke('subscribe_encoded_screenshot_updated', {
    displayId,
  });
}

export const ScreenView: Component<ScreenViewProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['displayId']);
  const [image, setImage] = createSignal<string>();
  createEffect(() => {
    const unlisten = listen<{ base64_image: string; display_id: number }>(
      'encoded-screenshot-updated',
      (event) => {
        if (event.payload.display_id === localProps.displayId) {
          setImage(event.payload.base64_image);
        }

        console.log(event.payload.display_id, localProps.displayId);
      },
    );
    subscribeScreenshotUpdate(localProps.displayId);

    onCleanup(() => {
      unlisten.then((unlisten) => {
        unlisten();
      });
    });
  });

  return <img src={image()} class="object-contain" {...rootProps} />;
};
