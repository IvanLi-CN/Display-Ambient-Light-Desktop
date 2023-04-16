import { listen } from '@tauri-apps/api/event';
import { createEffect, onCleanup } from 'solid-js';
import { ColorCalibration, LedStripConfigContainer } from '../../models/led-strip-config';
import { ledStripStore, setLedStripStore } from '../../stores/led-strip.store';
import { ColorSlider } from './color-slider';
import { TestColorsBg } from './test-colors-bg';
import { invoke } from '@tauri-apps/api';

export const WhiteBalance = () => {
  // listen to config_changed event
  createEffect(() => {
    const unlisten = listen('config_changed', (event) => {
      const { strips, mappers, color_calibration } =
        event.payload as LedStripConfigContainer;
      console.log(event.payload);
      setLedStripStore({
        strips,
        mappers,
        colorCalibration: color_calibration,
      });
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  const updateColorCalibration = (field: keyof ColorCalibration, value: number) => {
    const calibration = { ...ledStripStore.colorCalibration, [field]: value };
    console.log(field, calibration);
    invoke('set_color_calibration', {
      calibration,
    }).catch((error) => console.log(error));
  };

  const exit = () => {
    window.history.back();
  };

  return (
    <section class="select-none">
      <div class="absolute top-0 left-0 right-0 bottom-0">
        <TestColorsBg />
      </div>
      <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-10/12 max-w-lg bg-stone-200 p-5 rounded-xl drop-shadow">
        <label class="flex items-center gap-2">
          <span class="w-3 block">R:</span>
          <ColorSlider
            class="from-cyan-500 to-red-500"
            value={ledStripStore.colorCalibration.r}
            onInput={(ev) =>
              updateColorCalibration(
                'r',
                (ev.target as HTMLInputElement).valueAsNumber ?? 1,
              )
            }
          />
        </label>
        <label class="flex items-center gap-2">
          <span class="w-3 block">G:</span>
          <ColorSlider
            class="from-pink-500 to-green-500"
            value={ledStripStore.colorCalibration.g}
            onInput={(ev) =>
              updateColorCalibration(
                'g',
                (ev.target as HTMLInputElement).valueAsNumber ?? 1,
              )
            }
          />
        </label>
        <label class="flex items-center gap-2">
          <span class="w-3 block">B:</span>
          <ColorSlider
            class="from-yellow-500 to-blue-500"
            value={ledStripStore.colorCalibration.b}
            onInput={(ev) =>
              updateColorCalibration(
                'b',
                (ev.target as HTMLInputElement).valueAsNumber ?? 1,
              )
            }
          />
        </label>
        <label class="flex items-center gap-2">
          <span class="w-3 block">W:</span>
          <ColorSlider class="from-yellow-50 to-cyan-50" />
        </label>
        <button
          class="absolute -right-4 -top-4 rounded-full aspect-square bg-stone-300 p-1 shadow border border-stone-400"
          onClick={exit}
        >
          X
        </button>
        <button class="absolute -right-4 -bottom-4 rounded-full aspect-square bg-stone-300 p-1 shadow border border-stone-400">
          R
        </button>
      </div>
    </section>
  );
};
