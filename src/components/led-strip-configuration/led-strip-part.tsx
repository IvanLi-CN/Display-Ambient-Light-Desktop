import { invoke } from '@tauri-apps/api/core';
import {
  Component,
  createEffect,
  createMemo,
  createRoot,
  createSignal,
  For,
  JSX,
  splitProps,
  useContext,
} from 'solid-js';
import { useTippy } from 'solid-tippy';
import { followCursor } from 'tippy.js';
import { LedStripConfig } from '../../models/led-strip-config';
import { LedStripConfigurationContext } from '../../contexts/led-strip-configuration.context';
import { ledStripStore } from '../../stores/led-strip.store';

type LedStripPartProps = {
  config?: LedStripConfig | null;
} & JSX.HTMLAttributes<HTMLElement>;

type PixelProps = {
  color: string;
};

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
        class="absolute top-1/2 -translate-y-1/2 h-2.5 w-2.5 rounded-full ring-1 ring-stone-300/50"
        style={style()}
      />
    </div>
  );
};

export const LedStripPart: Component<LedStripPartProps> = (props) => {
  const [localProps, rootProps] = splitProps(props, ['config']);
  const [stripConfiguration, { setHoveredStripPart }] = useContext(LedStripConfigurationContext);

  const [colors, setColors] = createSignal<string[]>([]);

  // update led strip colors from global store
  createEffect(() => {
    if (!localProps.config) {
      return;
    }

    const strip = ledStripStore.strips.find(
      (s) =>
        s.display_id === localProps.config!.display_id &&
        s.border === localProps.config!.border,
    );

    if (!strip) {
      return;
    }

    // 基于 strip 的索引计算颜色偏移
    // 在新系统中，我们需要根据 strip 的索引来计算偏移
    const stripIndex = ledStripStore.strips.findIndex(s => s === strip);
    const offset = stripIndex * strip.len * 3;

    const colors = new Array(localProps.config.len).fill(null).map((_, i) => {
      const index = offset + i * 3;
      const r = ledStripStore.colors[index] || 0;
      const g = ledStripStore.colors[index + 1] || 0;
      const b = ledStripStore.colors[index + 2] || 0;
      return `rgb(${r}, ${g}, ${b})`;
    });

    setColors(colors);
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

  const onMouseEnter = () => {
    if (localProps.config) {
      setHoveredStripPart({
        displayId: localProps.config.display_id,
        border: localProps.config.border,
      });
    }
  };

  const onMouseLeave = () => {
    setHoveredStripPart(null);
  };

  return (
    <section
      {...rootProps}
      ref={setAnchor}
      class={
        'flex rounded-full flex-nowrap justify-around items-center overflow-hidden bg-gray-800/20 border border-gray-600/30 min-h-[12px] min-w-[12px] m-1 px-0.5 py-0.5 transition-all duration-200 ' +
        rootProps.class
      }
      classList={{
        'ring ring-inset bg-yellow-400/50 ring-orange-400 animate-pulse':
          stripConfiguration.selectedStripPart?.border === localProps.config?.border &&
          stripConfiguration.selectedStripPart?.displayId ===
            localProps.config?.display_id,
        'ring-2 ring-primary bg-primary/20 border-primary':
          stripConfiguration.hoveredStripPart?.border === localProps.config?.border &&
          stripConfiguration.hoveredStripPart?.displayId === localProps.config?.display_id,
      }}
      onWheel={onWheel}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
    >
      <For each={colors()}>{(item) => <Pixel color={item} />}</For>
    </section>
  );
};
