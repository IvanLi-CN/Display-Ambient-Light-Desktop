import { createEffect, createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { DisplayInfo } from './displays/display-info.model';

function App() {
  const [image, setImage] = createSignal<string>();
  const [screenshots, setScreenshots] = createSignal<string[]>([]);
  const [name, setName] = createSignal("");
  const [displays, setDisplays] = createSignal<DisplayInfo[]>([]);

  createEffect(() => {
    invoke<string>("list_display_info").then((displays) => {
      setDisplays(JSON.parse(displays));
    });
  });

  createEffect(() => {
    take_all_display_screenshot();
  }, [displays]);

  async function take_all_display_screenshot() {
    console.log("take_all_display_screenshot")
    displays().forEach((display) => {
      invoke<string>("take_screenshot", { displayId: display.id, scaleFactor: display.scale_factor }).then((image) => {
        setScreenshots((screenshots) => [...screenshots, image]);
      });
    });
  }

  return (
    <div class="container">
      <ol>
        {
          displays().map((display) => {
            return <li>
                <dl>
                  <dt>id</dt>
                  <dd>{display.id}</dd>
                </dl>
                <dl>
                  <dt>x</dt>
                  <dd>{display.x}</dd>
                </dl>
                <dl>
                  <dt>y</dt>
                  <dd>{display.y}</dd>
                </dl>
                <dl>
                  <dt>width</dt>
                  <dd>{display.width}</dd>
                </dl>
                <dl>
                  <dt>height</dt>
                  <dd>{display.height}</dd>
                </dl>
                <dl>
                  <dt>scale_factor</dt>
                  <dd>{display.scale_factor}</dd>
                </dl>
                <dl>
                  <dt>is_primary</dt>
                  <dd>{display.is_primary}</dd>
                </dl>
              </li>
          })
        }
      </ol>
      <ol>
        {
          screenshots().map((screenshot) => {
            return <li>
                <img style="object-fit: contain; height: 400px; width: 600px" src={screenshot}/>
              </li>
          })
        }
      </ol>
    </div>
  );
}

export default App;
