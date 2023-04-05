import {
  batch,
  Component,
  createContext,
  createEffect,
  createMemo,
  createSignal,
  For,
  Index,
  JSX,
  on,
  untrack,
  useContext,
} from 'solid-js';
import { LedStripConfig, LedStripPixelMapper } from '../models/led-strip-config';
import { ledStripStore } from '../stores/led-strip.store';
import { invoke } from '@tauri-apps/api';
import { LedStripConfigurationContext } from '../contexts/led-strip-configuration.context';
import background from '../assets/transparent-grid-background.svg?url';

const SorterItem: Component<{ strip: LedStripConfig; mapper: LedStripPixelMapper }> = (
  props,
) => {
  const [fullLeds, setFullLeds] = createSignal<Array<string | null>>([]);
  const [dragging, setDragging] = createSignal<boolean>(false);
  const [dragStart, setDragStart] = createSignal<{ x: number; y: number } | null>(null);
  const [dragCurr, setDragCurr] = createSignal<{ x: number; y: number } | null>(null);
  const [dragStartIndex, setDragStartIndex] = createSignal<number>(0);
  const [cellWidth, setCellWidth] = createSignal<number>(0);
  const [, { setSelectedStripPart }] = useContext(LedStripConfigurationContext);

  const move = (targetStart: number) => {
    if (targetStart === props.mapper.start) {
      return;
    }
    invoke('move_strip_part', {
      displayId: props.strip.display_id,
      border: props.strip.border,
      targetStart,
    }).catch((err) => console.error(err));
  };

  // reset translateX on config updated
  createEffect(() => {
    const indexDiff = props.mapper.start - dragStartIndex();
    untrack(() => {
      if (!dragStart() || !dragCurr()) {
        return;
      }

      const compensation = indexDiff * cellWidth();
      batch(() => {
        setDragStartIndex(props.mapper.start);
        setDragStart({
          x: dragStart()!.x + compensation,
          y: dragCurr()!.y,
        });
      });
    });
  });

  const onPointerDown = (ev: PointerEvent) => {
    if (ev.button !== 0) {
      return;
    }
    setDragging(true);
    setDragStart({ x: ev.clientX, y: ev.clientY });
    setDragCurr({ x: ev.clientX, y: ev.clientY });
    setDragStartIndex(props.mapper.start);
  };

  const onPointerUp = () => (ev: PointerEvent) => {
    if (ev.button !== 0) {
      return;
    }
    setDragging(false);
  };

  const onPointerMove = (ev: PointerEvent) => {
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

    const cellWidth =
      (ev.currentTarget as HTMLDivElement).clientWidth / ledStripStore.totalLedCount;
    const diff = ev.clientX - dragStart()!.x;
    const moved = Math.round(diff / cellWidth);
    if (moved === 0) {
      return;
    }
    setCellWidth(cellWidth);
    move(props.mapper.start + moved);
  };

  const onPointerLeave = () => {
    setSelectedStripPart(null);
  };

  const reverse = () => {
    invoke('reverse_led_strip_part', {
      displayId: props.strip.display_id,
      border: props.strip.border,
    }).catch((err) => console.error(err));
  };

  // update fullLeds
  createEffect(() => {
    const fullLeds = new Array(ledStripStore.totalLedCount).fill(null);
    const colors = ledStripStore.colors;

    const { start, end, pos } = props.mapper;
    const isForward = start < end;
    const step = isForward ? 1 : -1;
    for (let i = start, j = pos; i !== end; i += step, j++) {
      let c1 = `rgb(${Math.floor(colors[j * 3] * 0.8)}, ${Math.floor(
        colors[j * 3 + 1] * 0.8,
      )}, ${Math.floor(colors[j * 3 + 2] * 0.8)})`;
      let c2 = `rgb(${Math.min(Math.floor(colors[j * 3] * 1.2), 255)}, ${Math.min(
        Math.floor(colors[j * 3 + 1] * 1.2),
        255,
      )}, ${Math.min(Math.floor(colors[j * 3 + 2] * 1.2), 255)})`;

      fullLeds[i] = `linear-gradient(70deg, ${c1} 10%, ${c2})`;
    }

    setFullLeds(fullLeds);
  });

  const style = createMemo<JSX.CSSProperties>(() => {
    return {
      transform: `translateX(${(dragCurr()?.x ?? 0) - (dragStart()?.x ?? 0)}px)`,
    };
  });

  return (
    <div
      class="flex h-2 m-2 select-none cursor-ew-resize focus:cursor-ew-resize"
      style={style()}
      onPointerMove={onPointerMove}
      onPointerDown={onPointerDown}
      onPointerUp={onPointerUp}
      onPointerLeave={onPointerLeave}
      ondblclick={reverse}
    >
      <For each={fullLeds()}>
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
          <SorterItem strip={strip()} mapper={ledStripStore.mappers[index]} />
        )}
      </Index>
    </div>
  );
};
