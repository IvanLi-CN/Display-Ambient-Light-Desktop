import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  For,
  JSX,
  on,
  onCleanup,
  splitProps,
} from 'solid-js';
import { borders } from '../constants/border';
import { LedStripConfig } from '../models/led-strip-config';

type LedStripPartProps = {
  config?: LedStripConfig | null;
} & JSX.HTMLAttributes<HTMLElement>;

type PixelProps = {
  color: string;
};

async function subscribeScreenshotUpdate(displayId: number) {
  await invoke('subscribe_encoded_screenshot_updated', {
    displayId,
  });
}

export const Pixel: Component<PixelProps> = (props) => {
  const style = createMemo(() => ({
    background: props.color,
  }));
  return (
    <div
      class="flex-auto flex h-full w-full justify-center items-center relative"
      title={props.color}
    >
      <div
        class="absolute top-px h-2 w-2 rounded-full shadow-2xl shadow-black"
        style={style()}
      />
    </div>
  );
};

export const LedStripPart: Component<LedStripPartProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['config']);

  const [ledSamplePoints, setLedSamplePoints] = createSignal();
  const [colors, setColors] = createSignal<string[]>([]);

  createEffect(() => {
    const samplePoints = ledSamplePoints();
    if (!localProps.config || !samplePoints) {
      return;
    }

    let pendingCount = 0;
    const unlisten = listen<{
      base64_image: string;
      display_id: number;
      height: number;
      width: number;
    }>('encoded-screenshot-updated', (event) => {
      if (event.payload.display_id !== localProps.config!.display_id) {
        return;
      }
      if (pendingCount >= 1) {
        return;
      }
      pendingCount++;

      invoke<string[]>('get_one_edge_colors', {
        samplePoints,
        displayId: event.payload.display_id,
      })
        .then((colors) => {
          setColors(colors);
        })
        .finally(() => {
          pendingCount--;
        });
    });
    subscribeScreenshotUpdate(localProps.config.display_id);

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  createEffect(() => {
    if (localProps.config) {
      invoke('get_led_strips_sample_points', {
        config: localProps.config,
      }).then((points) => {
        console.log({ points });
        setLedSamplePoints(points);
      });
    }
  });

  const pixels = createMemo(() => {
    const _colors = colors();
    if (_colors) {
      return <For each={_colors}>{(item) => <Pixel color={item} />}</For>;
    } else if (localProps.config) {
      return (
        <For each={new Array(localProps.config.len).fill(undefined)}>
          {() => <Pixel color="transparent" />}
        </For>
      );
    }
  });

  return (
    <section
      {...rootProps}
      class={
        'bg-yellow-50 flex flex-nowrap justify-around items-center overflow-hidden' +
        rootProps.class
      }
    >
      {pixels()}
    </section>
  );
};
