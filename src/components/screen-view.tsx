import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { Component, createEffect, createSignal, onCleanup } from 'solid-js';

type ScreenViewProps = {
  displayId: number;
};

async function subscribeScreenshotUpdate(displayId: number) {
  await invoke('subscribe_encoded_screenshot_updated', {
    displayId,
  });
}

export const ScreenView: Component<ScreenViewProps> = (props) => {
  const [image, setImage] = createSignal<string>();
  createEffect(() => {
    const unlisten = listen<{ base64_image: string; display_id: number }>(
      'encoded-screenshot-updated',
      (event) => {
        if (event.payload.display_id === props.displayId) {
          setImage(event.payload.base64_image);
        }

        console.log(event.payload.display_id, props.displayId);
      },
    );
    subscribeScreenshotUpdate(props.displayId);

    onCleanup(() => {
      unlisten.then((unlisten) => {
        unlisten();
      });
    });
  });

  return <img src={image()} class="object-contain" />;
};
