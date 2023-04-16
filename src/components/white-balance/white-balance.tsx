import { listen } from '@tauri-apps/api/event';
import { Component, createEffect, onCleanup } from 'solid-js';
import { ColorCalibration, LedStripConfigContainer } from '../../models/led-strip-config';
import { ledStripStore, setLedStripStore } from '../../stores/led-strip.store';
import { ColorSlider } from './color-slider';
import { TestColorsBg } from './test-colors-bg';
import { invoke } from '@tauri-apps/api';
import { VsClose } from 'solid-icons/vs';
import { BiRegularReset } from 'solid-icons/bi';
import transparentBg from '../../assets/transparent-grid-background.svg?url';

const Value: Component<{ value: number }> = (props) => {
  return (
    <span class="w-10 text-sm block font-mono text-right ">
      {(props.value * 100).toFixed(0)}
      <span class="text-xs text-stone-600">%</span>
    </span>
  );
};

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
    invoke('set_color_calibration', {
      calibration,
    }).catch((error) => console.log(error));
  };

  const exit = () => {
    window.history.back();
  };

  const reset = () => {
    invoke('set_color_calibration', {
      calibration: new ColorCalibration(),
    }).catch((error) => console.log(error));
  };

  return (
    <section class="select-none text-stone-800">
      <div
        class="absolute top-0 left-0 right-0 bottom-0"
        style={{
          'background-image': `url(${transparentBg})`,
        }}
      >
        <TestColorsBg />
      </div>
      <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-10/12 max-w-lg bg-stone-100/20 backdrop-blur p-5 rounded-xl shadow-lg">
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
          <Value value={ledStripStore.colorCalibration.r} />
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
          <Value value={ledStripStore.colorCalibration.g} />
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
          <Value value={ledStripStore.colorCalibration.b} />
        </label>
        <label class="flex items-center gap-2">
          <span class="w-3 block">W:</span>
          <ColorSlider class="from-yellow-50 to-cyan-50" />
        </label>
        <button
          class="absolute -right-4 -top-4 rounded-full aspect-square bg-stone-100/20 backdrop-blur p-1 shadow hover:bg-stone-200/20 active:bg-stone-300"
          onClick={exit}
          title="Go Back"
        >
          <VsClose size={24} />
        </button>
        <button
          class="absolute -right-4 -bottom-4 rounded-full aspect-square bg-stone-100/20 backdrop-blur p-1 shadow hover:bg-stone-200/20 active:bg-stone-300"
          onClick={reset}
          title="Reset to 100%"
        >
          <BiRegularReset size={24} />
        </button>
      </div>
    </section>
  );
};
