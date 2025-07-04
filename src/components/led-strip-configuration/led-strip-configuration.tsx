import { createEffect, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
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
      const parsedDisplays = JSON.parse(displays);
      console.log('LedStripConfiguration: Loaded displays:', parsedDisplays);
      setDisplayStore({
        displays: parsedDisplays,
      });
    }).catch((error) => {
      console.error('LedStripConfiguration: Failed to load displays:', error);
    });

    invoke<LedStripConfigContainer>('read_led_strip_configs').then((configs) => {
      console.log('LedStripConfiguration: Loaded LED strip configs:', configs);
      setLedStripStore(configs);
    }).catch((error) => {
      console.error('LedStripConfiguration: Failed to load LED strip configs:', error);
    });
  });

  // listen to config_changed event
  createEffect(() => {
    const unlisten = listen('config_changed', (event) => {
      const { strips, mappers } = event.payload as LedStripConfigContainer;
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
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-base-content">灯条配置</h1>
        <div class="stats shadow">
          <div class="stat">
            <div class="stat-title">显示器数量</div>
            <div class="stat-value text-primary">{displayStore.displays.length}</div>
          </div>
        </div>
      </div>

      <LedStripConfigurationContext.Provider value={ledStripConfigurationContextValue}>
        {/* LED Strip Sorter Panel */}
        <div class="card bg-base-200 shadow-lg">
          <div class="card-body p-4">
            <div class="card-title text-base mb-3">
              <span>灯条排序</span>
              <div class="badge badge-info badge-outline">实时预览</div>
            </div>
            <LedStripPartsSorter />
            <div class="text-xs text-base-content/50 mt-2">
              💡 提示：拖拽灯条段落来调整顺序，双击可反转方向
            </div>
          </div>
        </div>

        {/* Display Configuration Panel */}
        <div class="card bg-base-200 shadow-lg">
          <div class="card-body p-4">
            <div class="card-title text-base mb-3">
              <span>显示器配置</span>
              <div class="badge badge-secondary badge-outline">可视化编辑</div>
            </div>
            <DisplayListContainer>
              {displayStore.displays.map((display) => {
                console.log('LedStripConfiguration: Rendering DisplayView for display:', display);
                return <DisplayView display={display} />;
              })}
            </DisplayListContainer>
            <div class="text-xs text-base-content/50 mt-2">
              💡 提示：鼠标滚轮调整灯条长度，悬停查看详细信息
            </div>
          </div>
        </div>
      </LedStripConfigurationContext.Provider>
    </div>
  );
};
