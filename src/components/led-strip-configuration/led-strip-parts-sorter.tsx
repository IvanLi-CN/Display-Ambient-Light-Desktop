import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  For,
  Index,
  JSX,
  Match,
  onCleanup,
  onMount,
  Switch,
  useContext,
} from 'solid-js';
import { LedStripConfig } from '../../models/led-strip-config';
import { ledStripStore } from '../../stores/led-strip.store';
import { LedStripConfigurationContext } from '../../contexts/led-strip-configuration.context';
import background from '../../assets/transparent-grid-background.svg?url';

const SorterItem: Component<{ strip: LedStripConfig; stripIndex: number }> = (
  props,
) => {
  const [leds, setLeds] = createSignal<Array<string | null>>([]);
  const [stripConfiguration, { setSelectedStripPart, setHoveredStripPart }] = useContext(LedStripConfigurationContext);
  const [rootWidth, setRootWidth] = createSignal<number>(0);
  const [cellWidth, setCellWidth] = createSignal<number>(0);

  let root: HTMLDivElement | undefined;

  // 计算当前 strip 在全局 LED 数组中的起始位置
  const getStripStartPosition = () => {
    let position = 0;
    for (let i = 0; i < props.stripIndex; i++) {
      position += ledStripStore.strips[i]?.len || 0;
    }
    return position;
  };





  const onPointerLeave = () => {
    setSelectedStripPart(null);
  };



  const onMouseEnter = () => {
    setHoveredStripPart({
      displayId: props.strip.display_id,
      border: props.strip.border,
    });
  };

  const onMouseLeave = () => {
    setHoveredStripPart(null);
  };

  const setColor = (fullIndex: number, colorsIndex: number, fullLeds: string[]) => {
    // 使用新的 assembledColors 计算属性，它会优先使用按灯带分组的颜色数据
    const colors = ledStripStore.assembledColors;
    let c1 = `rgb(${Math.floor(colors[colorsIndex * 3] * 0.8)}, ${Math.floor(
      colors[colorsIndex * 3 + 1] * 0.8,
    )}, ${Math.floor(colors[colorsIndex * 3 + 2] * 0.8)})`;
    let c2 = `rgb(${Math.min(Math.floor(colors[colorsIndex * 3] * 1.2), 255)}, ${Math.min(
      Math.floor(colors[colorsIndex * 3 + 1] * 1.2),
      255,
    )}, ${Math.min(Math.floor(colors[colorsIndex * 3 + 2] * 1.2), 255)})`;

    if (fullLeds.length <= fullIndex) {
      return;
    }

    fullLeds[fullIndex] = `linear-gradient(70deg, ${c1} 10%, ${c2})`;
  };

  // update fullLeds
  createEffect(() => {
    const stripStartPos = getStripStartPosition();
    const stripLength = props.strip.len;

    const leds = new Array(stripLength).fill(null);

    // 简化逻辑：直接按顺序设置颜色
    for (let i = 0; i < leds.length; i++) {
      setColor(i, stripStartPos + i, leds);
    }

    setLeds(leds);
  });

  // update rootWidth
  onMount(() => {
    if (!root) return;

    let observer: ResizeObserver;
    observer = new ResizeObserver(() => {
      if (root) {
        setRootWidth(root.clientWidth);
      }
    });
    observer.observe(root);

    onCleanup(() => {
      if (root) {
        observer?.unobserve(root);
      }
    });
  });

  // update cellWidth
  createEffect(() => {
    const cellWidth = rootWidth() / ledStripStore.totalLedCount;
    setCellWidth(cellWidth);
  });

  const style = createMemo<JSX.CSSProperties>(() => {
    const stripStartPos = getStripStartPosition();
    return {
      transform: `translateX(${cellWidth() * stripStartPos}px)`,
      width: `${cellWidth() * leds().length}px`,
    };
  });

  return (
    <div
      class="flex mx-2 select-none transition-colors duration-200"
      classList={{
        'bg-primary/20 rounded-lg':
          stripConfiguration.hoveredStripPart?.border === props.strip.border &&
          stripConfiguration.hoveredStripPart?.displayId === props.strip.display_id,
      }}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      ref={root!}
    >
      <div
        style={style()}
        class="rounded-full border border-white flex h-3 bg-gradient-to-b from-white/50 to-stone-500/40"
      >
        <For each={leds()}>
          {(it) => (
            <div
              class="flex-auto flex h-full w-full justify-center items-center relative"
              title={it ?? ''}
            >
              <div
                class="absolute top-1/2 -translate-y-1/2 h-2.5 w-2.5 rounded-full ring-1 ring-stone-100"
                classList={{ 'ring-stone-300/50': !it }}
                style={{ background: it ?? 'transparent' }}
              />
            </div>
          )}
        </For>
      </div>
    </div>
  );
};

const SorterResult: Component = () => {
  const [fullLeds, setFullLeds] = createSignal<string[]>([]);

  createEffect(() => {
    const colors = ledStripStore.assembledColors;
    const fullLeds = new Array(ledStripStore.totalLedCount)
      .fill('rgba(255,255,255,0.1)')
      .map((_, i) => {
        let c1 = `rgb(${Math.floor(colors[i * 3] * 0.8)}, ${Math.floor(
          colors[i * 3 + 1] * 0.8,
        )}, ${Math.floor(colors[i * 3 + 2] * 0.8)})`;
        let c2 = `rgb(${Math.min(Math.floor(colors[i * 3] * 1.2), 255)}, ${Math.min(
          Math.floor(colors[i * 3 + 1] * 1.2),
          255,
        )}, ${Math.min(Math.floor(colors[i * 3 + 2] * 1.2), 255)})`;

        return `linear-gradient(70deg, ${c1} 10%, ${c2})`;
      });
    setFullLeds(fullLeds);
  });

  return (
    <div class="flex h-2 m-2">
      <For each={fullLeds()}>
        {(it) => (
          <div
            class="flex-auto flex h-full w-full justify-center items-center relative"
            title={it}
          >
            <div
              class="absolute top-1/2 -translate-y-1/2 h-2.5 w-2.5 rounded-full ring-1 ring-stone-300"
              style={{ background: it }}
            />
          </div>
        )}
      </For>
    </div>
  );
};

export const LedStripPartsSorter: Component = () => {
  return (
    <div
      class="select-none overflow-hidden"
      style={{
        'background-image': `url(${background})`,
      }}
    >
      <SorterResult />
      <Index each={ledStripStore.strips}>
        {(strip, index) => (
          <Switch>
            <Match when={strip().len > 0}>
              <SorterItem strip={strip()} stripIndex={index} />
            </Match>
          </Switch>
        )}
      </Index>
    </div>
  );
};
