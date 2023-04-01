import {
  Component,
  createContext,
  createEffect,
  createMemo,
  createSignal,
  For,
  JSX,
  useContext,
} from 'solid-js';
import { LedStripConfig, LedStripPixelMapper } from '../models/led-strip-config';
import { ledStripStore } from '../stores/led-strip.store';
import { invoke } from '@tauri-apps/api';
import { LedStripConfigurationContext } from '../contexts/led-strip-configuration.context';

const SorterItem: Component<{ strip: LedStripConfig; mapper: LedStripPixelMapper }> = (
  props,
) => {
  const [fullLeds, setFullLeds] = createSignal<string[]>([]);
  const [dragging, setDragging] = createSignal<boolean>(false);
  const [dragStart, setDragStart] = createSignal<{ x: number; y: number } | null>(null);
  const [dragCurr, setDragCurr] = createSignal<{ x: number; y: number } | null>(null);
  const [dragStartIndex, setDragStartIndex] = createSignal<number>(0);
  const [, { setSelectedStripPart }] = useContext(LedStripConfigurationContext);

  const totalLedCount = createMemo(() => {
    return ledStripStore.strips.reduce((acc, strip) => acc + strip.len, 0);
  });

  const move = (targetStart: number) => {
    if (targetStart === props.mapper.start) {
      return;
    }
    console.log(`target_start ${targetStart}`);
    invoke('move_strip_part', {
      displayId: props.strip.display_id,
      border: props.strip.border,
      targetStart,
    }).catch((err) => console.error(err));
  };

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

    const cellWidth = (ev.currentTarget as HTMLDivElement).clientWidth / totalLedCount();
    const diff = ev.clientX - dragStart()!.x;
    const moved = Math.round(diff / cellWidth);
    if (moved === 0) {
      return;
    }
    move(props.mapper.start + moved);
  };

  const onPointerLeave = () => {
    setSelectedStripPart(null);
  };

  // update fullLeds
  createEffect(() => {
    const fullLeds = new Array(totalLedCount()).fill('rgba(255,255,255,0.5)');

    for (let i = props.mapper.start, j = 0; i < props.mapper.end; i++, j++) {
      fullLeds[i] = `rgb(${ledStripStore.colors[i * 3]}, ${
        ledStripStore.colors[i * 3 + 1]
      }, ${ledStripStore.colors[i * 3 + 2]})`;
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
    >
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

const SorterResult: Component = () => {
  const [fullLeds, setFullLeds] = createSignal<string[]>([]);

  createEffect(() => {
    const strips = ledStripStore.strips;
    const totalLedCount = strips.reduce((acc, strip) => acc + strip.len, 0);
    const fullLeds = new Array(totalLedCount).fill('rgba(255,255,255,0.5)');

    ledStripStore.mappers.forEach((mapper) => {
      for (let i = mapper.start, j = 0; i < mapper.end; i++, j++) {
        fullLeds[i] = `rgb(${ledStripStore.colors[i * 3]}, ${
          ledStripStore.colors[i * 3 + 1]
        }, ${ledStripStore.colors[i * 3 + 2]})`;
      }
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
  const context = createContext();

  return (
    <div class="select-none overflow-hidden">
      <SorterResult />
      <For each={ledStripStore.strips}>
        {(strip, index) => (
          <SorterItem strip={strip} mapper={ledStripStore.mappers[index()]} />
        )}
      </For>
    </div>
  );
};
