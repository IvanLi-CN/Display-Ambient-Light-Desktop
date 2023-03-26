import { createEffect, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri';
import { DisplayView } from './components/display-view';
import { DisplayListContainer } from './components/display-list-container';
import { displayStore, setDisplayStore } from './stores/display.store';
import { LedStripConfig } from './models/led-strip-config';
import { setLedStripStore } from './stores/led-strip.store';
import { listen } from '@tauri-apps/api/event';

function App() {
  createEffect(() => {
    invoke<string>('list_display_info').then((displays) => {
      setDisplayStore({
        displays: JSON.parse(displays),
      });
    });
    invoke<LedStripConfig[]>('read_led_strip_configs').then((strips) => {
      setLedStripStore({
        strips,
      });
    });
  });

  // register tauri event listeners
  createEffect(() => {
    const unlisten = listen('config_changed', (event) => {
      const strips = event.payload as LedStripConfig[];
      setLedStripStore({
        strips,
      });
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  return (
    <div>
      <DisplayListContainer>
        {displayStore.displays.map((display) => {
          return <DisplayView display={display} />;
        })}
      </DisplayListContainer>
    </div>
  );
}

export default App;
