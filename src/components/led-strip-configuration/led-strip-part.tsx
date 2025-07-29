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
  colors?: Uint8ClampedArray; // 从父组件接收颜色数据
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
  const [localProps, rootProps] = splitProps(props, ['config', 'colors']);
  const [stripConfiguration, { setHoveredStripPart }] = useContext(LedStripConfigurationContext);

  const [colors, setColors] = createSignal<string[]>([]);

  // 使用父组件传递的颜色数据
  createEffect(() => {
    if (!localProps.config) {
      return;
    }

    // 默认颜色函数
    const getDefaultColor = (border: string) => {
      const colorMap = {
        'Top': 'linear-gradient(70deg, rgb(100, 150, 255) 10%, rgb(200, 230, 255))',
        'Bottom': 'linear-gradient(70deg, rgb(255, 150, 100) 10%, rgb(255, 230, 200))',
        'Left': 'linear-gradient(70deg, rgb(150, 255, 100) 10%, rgb(230, 255, 200))',
        'Right': 'linear-gradient(70deg, rgb(255, 100, 150) 10%, rgb(255, 200, 230))',
      };
      return colorMap[border as keyof typeof colorMap] || 'linear-gradient(70deg, rgb(128, 128, 128) 10%, rgb(200, 200, 200))';
    };

    // 如果没有颜色数据，使用默认颜色
    if (!localProps.colors || localProps.colors.length === 0) {
      const defaultColor = getDefaultColor(localProps.config.border);
      setColors(new Array(localProps.config.len).fill(defaultColor));
      return;
    }

    // 检查数据是否足够
    const requiredBytes = localProps.config.len * 3;
    if (localProps.colors.length < requiredBytes) {
      const defaultColor = getDefaultColor(localProps.config.border);
      setColors(new Array(localProps.config.len).fill(defaultColor));
      return;
    }

    // 从颜色数据中提取RGB值
    const newColors = new Array(localProps.config.len).fill(null).map((_, i) => {
      const index = i * 3;
      const r = localProps.colors![index] || 0;
      const g = localProps.colors![index + 1] || 0;
      const b = localProps.colors![index + 2] || 0;
      return `rgb(${r}, ${g}, ${b})`;
    });

    setColors(newColors);
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
