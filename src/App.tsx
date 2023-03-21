import { createEffect } from 'solid-js';
import { convertFileSrc, invoke } from '@tauri-apps/api/tauri';
import { DisplayView } from './components/display-view';
import { DisplayListContainer } from './components/display-list-container';
import { displayStore, setDisplayStore } from './stores/display.store';
import { path } from '@tauri-apps/api';

function App() {
  createEffect(() => {
    invoke<string>('list_display_info').then((displays) => {
      setDisplayStore({
        displays: JSON.parse(displays),
      });
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
