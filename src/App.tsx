import { createEffect } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri';
import { DisplayView } from './components/\u0016display-view';
import { DisplayListContainer } from './components/display-list-container';
import { displayStore, setDisplayStore } from './stores/display.store';

function App() {
  createEffect(() => {
    invoke<string>('list_display_info').then((displays) => {
      setDisplayStore({
        displays: JSON.parse(displays),
      });
    });
  });

  return (
    <div class="container">
      <DisplayListContainer>
        {displayStore.displays.map((display) => {
          return <DisplayView display={display} />;
        })}
      </DisplayListContainer>
    </div>
  );
}

export default App;
