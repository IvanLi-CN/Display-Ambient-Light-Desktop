import { createEffect, createSignal } from "solid-js";
import { invoke } from '@tauri-apps/api/tauri';
import { DisplayInfo } from './models/display-info.model';
import { DisplayView } from './components/\u0016display-view';

function App() {
  const [displays, setDisplays] = createSignal<DisplayInfo[]>([]);

  createEffect(() => {
    invoke<string>('list_display_info').then((displays) => {
      setDisplays(JSON.parse(displays));
    });
  });

  return (
    <div class="container">
      <ol>
        {displays().map((display) => {
          return <DisplayView display={display} />;
        })}
      </ol>
    </div>
  );
}

export default App;
