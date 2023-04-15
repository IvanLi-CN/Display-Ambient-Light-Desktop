import {
  batch,
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
  untrack,
  useContext,
} from 'solid-js';
import { LedStripConfig, LedStripPixelMapper } from '../../models/led-strip-config';
import { ledStripStore } from '../../stores/led-strip.store';
import { invoke } from '@tauri-apps/api';
import { LedStripConfigurationContext } from '../../contexts/led-strip-configuration.context';
import background from '../../assets/transparent-grid-background.svg?url';

const SorterItem: Component<{ strip: LedStripConfig; mapper: LedStripPixelMapper }> = (
  props,
) => {
  const [leds, setLeds] = createSignal<Array<string | null>>([]);
  const [dragging, setDragging] = createSignal<boolean>(false);
  const [dragStart, setDragStart] = createSignal<{ x: number; y: number } | null>(null);
  const [dragCurr, setDragCurr] = createSignal<{ x: number; y: number } | null>(null);
  const [dragStartIndex, setDragStartIndex] = createSignal<number>(0);
  const [cellWidth, setCellWidth] = createSignal<number>(0);
  const [, { setSelectedStripPart }] = useContext(LedStripConfigurationContext);
  const [rootWidth, setRootWidth] = createSignal<number>(0);

  let root: HTMLDivElement;

  const move = (targetStart: number) => {
    if (targetStart === props.mapper.start) {
      return;
    }
    console.log(
      `moving strip part ${props.strip.display_id} ${props.strip.border} from ${props.mapper.start} to ${targetStart}`,
    );
    invoke('move_strip_part', {
      displayId: props.strip.display_id,
      border: props.strip.border,
      targetStart,
    }).catch((err) => console.error(err));
  };

  // reset translateX on config updated
  createEffect(() => {
    const indexDiff = props.mapper.start - dragStartIndex();
    const start = untrack(dragStart);
    const curr = untrack(dragCurr);
    const _dragging = untrack(dragging);

    if (start === null || curr === null) {
      return;
    }
    if (_dragging && indexDiff !== 0) {
      const compensation = indexDiff * cellWidth();
      batch(() => {
        setDragStartIndex(props.mapper.start);
        setDragStart({
          x: start.x + compensation,
          y: curr.y,
        });
      });
    } else {
      batch(() => {
        setDragStartIndex(props.mapper.start);
        setDragStart(null);
        setDragCurr(null);
      });
    }
  });

  const onPointerDown = (ev: PointerEvent) => {
    if (ev.button !== 0) {
      return;
    }
    batch(() => {
      setDragging(true);
      if (dragStart() === null) {
        setDragStart({ x: ev.clientX, y: ev.clientY });
      }
      setDragCurr({ x: ev.clientX, y: ev.clientY });
      setDragStartIndex(props.mapper.start);
    });
  };

  const onPointerUp = (ev: PointerEvent) => {
    if (ev.button !== 0) {
      return;
    }

    if (dragging() === false) {
      return;
    }
    setDragging(false);
    const diff = ev.clientX - dragStart()!.x;
    const moved = Math.round(diff / cellWidth());
    if (moved === 0) {
      return;
    }
    move(props.mapper.start + moved);
  };

  const onPointerMove = (ev: PointerEvent) => {
    if (dragging() === false) {
      return;
    }

    setSelectedStripPart({
      displayId: props.strip.display_id,
      border: props.strip.border,
    });
    if (!(ev.buttons & 1)) {
      return;
    }
    const draggingInfo = dragging();
    if (!draggingInfo) {
      return;
    }
    setDragCurr({ x: ev.clientX, y: ev.clientY });
  };

  const onPointerLeave = () => {
    setSelectedStripPart(null);
  };

  createEffect(() => {
    onMount(() => {
      window.addEventListener('pointermove', onPointerMove);
      window.addEventListener('pointerleave', onPointerLeave);
      window.addEventListener('pointerup', onPointerUp);
    });

    onCleanup(() => {
      window.removeEventListener('pointermove', onPointerMove);
      window.removeEventListener('pointerleave', onPointerLeave);
      window.removeEventListener('pointerup', onPointerUp);
    });
  });

  const reverse = () => {
    invoke('reverse_led_strip_part', {
      displayId: props.strip.display_id,
      border: props.strip.border,
    }).catch((err) => console.error(err));
  };

  const setColor = (fullIndex: number, colorsIndex: number, fullLeds: string[]) => {
    const colors = ledStripStore.colors;
    let c1 = `rgb(${Math.floor(colors[colorsIndex * 3] * 0.8)}, ${Math.floor(
      colors[colorsIndex * 3 + 1] * 0.8,
    )}, ${Math.floor(colors[colorsIndex * 3 + 2] * 0.8)})`;
    let c2 = `rgb(${Math.min(Math.floor(colors[colorsIndex * 3] * 1.2), 255)}, ${Math.min(
      Math.floor(colors[colorsIndex * 3 + 1] * 1.2),
      255,
    )}, ${Math.min(Math.floor(colors[colorsIndex * 3 + 2] * 1.2), 255)})`;

    if (fullLeds.length <= fullIndex) {
      console.error('out of range', fullIndex, fullLeds.length);
      return;
    }

    fullLeds[fullIndex] = `linear-gradient(70deg, ${c1} 10%, ${c2})`;
  };

  // update fullLeds
  createEffect(() => {
    const { start, end, pos } = props.mapper;

    const leds = new Array(Math.abs(start - end)).fill(null);

    if (start < end) {
      for (let i = 0, j = pos; i < leds.length; i++, j++) {
        setColor(i, j, leds);
      }
    } else {
      for (let i = leds.length - 1, j = pos; i >= 0; i--, j++) {
        setColor(i, j, leds);
      }
    }

    setLeds(leds);
  });

  // update rootWidth
  createEffect(() => {
    let observer: ResizeObserver;
    onMount(() => {
      observer = new ResizeObserver(() => {
        setRootWidth(root.clientWidth);
      });
      observer.observe(root);
    });

    onCleanup(() => {
      observer?.unobserve(root);
    });
  });
  // update cellWidth
  createEffect(() => {
    const cellWidth = rootWidth() / ledStripStore.totalLedCount;
    setCellWidth(cellWidth);
  });

  const style = createMemo<JSX.CSSProperties>(() => {
    return {
      transform: `translateX(${
        (dragCurr()?.x ?? 0) -
        (dragStart()?.x ?? 0) +
        cellWidth() * Math.min(props.mapper.start, props.mapper.end)
      }px)`,
      width: `${cellWidth() * leds().length}px`,
    };
  });

  return (
    <div
      class="flex mx-2 select-none cursor-ew-resize focus:cursor-ew-resize"
      onPointerDown={onPointerDown}
      ondblclick={reverse}
      ref={root!}
    >
      <div
        style={style()}
        class="rounded-full border border-white flex h-3"
        classList={{
          'bg-gradient-to-b from-yellow-500/60 to-orange-300/60': dragging(),
          'bg-gradient-to-b from-white/50 to-stone-500/40': !dragging(),
        }}
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
    const colors = ledStripStore.sortedColors;
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
              <SorterItem strip={strip()} mapper={ledStripStore.mappers[index]} />
            </Match>
          </Switch>
        )}
      </Index>
    </div>
  );
};
