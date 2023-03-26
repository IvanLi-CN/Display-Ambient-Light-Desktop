import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import {
  Component,
  createEffect,
  createMemo,
  createRoot,
  createSignal,
  For,
  JSX,
  onCleanup,
  splitProps,
} from 'solid-js';
import { useTippy } from 'solid-tippy';
import { followCursor } from 'tippy.js';
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
        class="absolute top-1/2 -translate-y-1/2 h-2.5 w-2.5 rounded-full ring-1 ring-stone-300"
        style={style()}
      />
    </div>
  );
};

export const LedStripPart: Component<LedStripPartProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['config']);

  const [ledSamplePoints, setLedSamplePoints] = createSignal();
  const [colors, setColors] = createSignal<string[]>([]);

  // get led strip colors when screenshot updated
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

  // get led strip sample points
  createEffect(() => {
    if (localProps.config) {
      invoke('get_led_strips_sample_points', {
        config: localProps.config,
      }).then((points) => {
        setLedSamplePoints(points);
      });
    }
  });

  const [anchor, setAnchor] = createSignal<HTMLElement>();

  useTippy(anchor, {
    hidden: true,
    props: {
      trigger: 'mouseenter focus',
      followCursor: true,

      plugins: [followCursor],

      content: () =>
        createRoot(() => {
          return (
            <span class="rounded-lg bg-slate-400/50 backdrop-blur text-white p-2 drop-shadow">
              Count: {localProps.config?.len ?? '--'}
            </span>
          );
        }) as Element,
    },
  });

  const onWheel = (e: WheelEvent) => {
    if (localProps.config) {
      invoke('patch_led_strip_len', {
        displayId: localProps.config.display_id,
        border: localProps.config.border,
        deltaLen: e.deltaY > 0 ? 1 : -1,
      })
        .then(() => {})
        .catch((e) => {
          console.error(e);
        });
    }
  };

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
      ref={setAnchor}
      class={
        'flex flex-nowrap justify-around items-center overflow-hidden ' + rootProps.class
      }
      onWheel={onWheel}
    >
      {pixels()}
    </section>
  );
};
