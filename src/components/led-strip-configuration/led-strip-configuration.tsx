import { createEffect, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri';
import { DisplayView } from './display-view';
import { DisplayListContainer } from './display-list-container';
import { displayStore, setDisplayStore } from '../../stores/display.store';
import { LedStripConfigContainer } from '../../models/led-strip-config';
import { setLedStripStore } from '../../stores/led-strip.store';
import { listen } from '@tauri-apps/api/event';
import { LedStripPartsSorter } from './led-strip-parts-sorter';
import { createStore } from 'solid-js/store';
import {
  LedStripConfigurationContext,
  LedStripConfigurationContextType,
} from '../../contexts/led-strip-configuration.context';

export const LedStripConfiguration = () => {
  createEffect(() => {
    invoke<string>('list_display_info').then((displays) => {
      setDisplayStore({
        displays: JSON.parse(displays),
      });
    });
    invoke<LedStripConfigContainer>('read_led_strip_configs').then((configs) => {
      console.log(configs);
      setLedStripStore(configs);
    });
  });

  // listen to config_changed event
  createEffect(() => {
    const unlisten = listen('config_changed', (event) => {
      const { strips, mappers } = event.payload as LedStripConfigContainer;
      console.log(event.payload);
      setLedStripStore({
        strips,
        mappers,
      });
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  // listen to led_colors_changed event
  createEffect(() => {
    const unlisten = listen<Uint8ClampedArray>('led_colors_changed', (event) => {
      if (!window.document.hidden) {
        const colors = event.payload;
        setLedStripStore({
          colors,
        });
      }
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  // listen to led_sorted_colors_changed event
  createEffect(() => {
    const unlisten = listen<Uint8ClampedArray>('led_sorted_colors_changed', (event) => {
      if (!window.document.hidden) {
        const sortedColors = event.payload;
        setLedStripStore({
          sortedColors,
        });
      }
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  const [ledStripConfiguration, setLedStripConfiguration] = createStore<
    LedStripConfigurationContextType[0]
  >({
    selectedStripPart: null,
  });

  const ledStripConfigurationContextValue: LedStripConfigurationContextType = [
    ledStripConfiguration,
    {
      setSelectedStripPart: (v) => {
        setLedStripConfiguration({
          selectedStripPart: v,
        });
      },
    },
  ];

  return (
    <div>
      <LedStripConfigurationContext.Provider value={ledStripConfigurationContextValue}>
        <LedStripPartsSorter />
        <DisplayListContainer>
          {displayStore.displays.map((display) => {
            return <DisplayView display={display} />;
          })}
        </DisplayListContainer>
      </LedStripConfigurationContext.Provider>
    </div>
  );
};
