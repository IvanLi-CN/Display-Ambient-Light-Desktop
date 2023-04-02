import { createContext, createEffect, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri';
import { DisplayView } from './components/display-view';
import { DisplayListContainer } from './components/display-list-container';
import { displayStore, setDisplayStore } from './stores/display.store';
import { LedStripConfigContainer } from './models/led-strip-config';
import { setLedStripStore } from './stores/led-strip.store';
import { listen } from '@tauri-apps/api/event';
import { LedStripPartsSorter } from './components/led-strip-parts-sorter';
import { createStore } from 'solid-js/store';
import {
  LedStripConfigurationContext,
  LedStripConfigurationContextType,
} from './contexts/led-strip-configuration.context';

function App() {
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
    const unlisten = listen<Array<string>>('led_colors_changed', (event) => {
      const colors = event.payload;

      setLedStripStore({
        colors,
      });
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  // listen to led_sorted_colors_changed event
  createEffect(() => {
    const unlisten = listen<Uint8ClampedArray>('led_sorted_colors_changed', (event) => {
      const sortedColors = event.payload;

      setLedStripStore({
        sortedColors,
      });
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
}

export default App;
