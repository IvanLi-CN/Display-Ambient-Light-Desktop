import { createEffect, createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { DisplayInfo } from './models/display-info.model';
import { DisplayInfoPanel } from './components/display-info-panel';
import { ScreenView } from './components/screen-view';

function App() {
  const [screenshots, setScreenshots] = createSignal<string[]>([]);
  const [displays, setDisplays] = createSignal<DisplayInfo[]>([]);

  createEffect(() => {
    invoke<string>('list_display_info').then((displays) => {
      setDisplays(JSON.parse(displays));
    });
  });

  // createEffect(() => {
  //   take_all_display_screenshot();
  // }, [displays]);

  async function take_all_display_screenshot() {
    console.log('take_all_display_screenshot');
    displays().forEach((display) => {
      invoke<string>('take_screenshot', {
        displayId: display.id,
        scaleFactor: display.scale_factor,
      }).then((image) => {
        setScreenshots((screenshots) => [...screenshots, image]);
      });
    });
  }

  return (
    <div class="container">
      <ol>
        {displays().map((display) => {
          return (
            <li>
              <DisplayInfoPanel display={display} />
              <ScreenView displayId={display.id} />
            </li>
          );
        })}
      </ol>
    </div>
  );
}

export default App;
