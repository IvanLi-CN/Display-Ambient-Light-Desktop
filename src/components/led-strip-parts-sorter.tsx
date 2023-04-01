import { Component, createContext, createEffect, createSignal, For } from 'solid-js';
import { LedStripConfig, LedStripPixelMapper } from '../models/led-strip-config';
import { ledStripStore } from '../stores/led-strip.store';

const SorterItem: Component<{ mapper: LedStripPixelMapper; strip: LedStripConfig }> = (
  props,
) => {
  const [fullLeds, setFullLeds] = createSignal<string[]>([]);

  createEffect(() => {
    const strips = ledStripStore.strips;
    const totalLedCount = strips.reduce((acc, strip) => acc + strip.len, 0);
    const fullLeds = new Array(totalLedCount).fill('rgba(255,255,255,0.5)');

    for (let i = props.mapper.start, j = 0; i < props.mapper.end; i++, j++) {
      fullLeds[i] = `rgb(${ledStripStore.colors[i * 3]}, ${
        ledStripStore.colors[i * 3 + 1]
      }, ${ledStripStore.colors[i * 3 + 2]})`;
    }
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
    <div>
      <For each={ledStripStore.strips}>
        {(strip, index) => (
          <SorterItem strip={strip} mapper={ledStripStore.mappers[index()]} />
        )}
      </For>
    </div>
  );
};
