import { createEffect, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { DisplayView } from './display-view';
import { DisplayListContainer } from './display-list-container';
import { displayStore, setDisplayStore } from '../../stores/display.store';
import { LedStripConfigContainer } from '../../models/led-strip-config';
import { setLedStripStore } from '../../stores/led-strip.store';
import { listen } from '@tauri-apps/api/event';
import { LedStripPartsSorter } from './led-strip-parts-sorter';
import { LedCountControlPanel } from './led-count-control-panel';
import { createStore } from 'solid-js/store';
import {
  LedStripConfigurationContext,
  LedStripConfigurationContextType,
} from '../../contexts/led-strip-configuration.context';


export const LedStripConfiguration = () => {
  createEffect(() => {
    invoke<string>('list_display_info').then((displays) => {
      const parsedDisplays = JSON.parse(displays);
      setDisplayStore({
        displays: parsedDisplays,
      });
    }).catch((error) => {
      console.error('Failed to load displays:', error);
    });

    invoke<LedStripConfigContainer>('read_led_strip_configs').then((configs) => {
      setLedStripStore(configs);
    }).catch((error) => {
      console.error('Failed to load LED strip configs:', error);
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
    hoveredStripPart: null,
  });

  const ledStripConfigurationContextValue: LedStripConfigurationContextType = [
    ledStripConfiguration,
    {
      setSelectedStripPart: (v) => {
        setLedStripConfiguration({
          selectedStripPart: v,
        });
      },
      setHoveredStripPart: (v) => {
        setLedStripConfiguration({
          hoveredStripPart: v,
        });
      },
    },
  ];

  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h1 class="text-xl font-bold text-base-content">灯条配置</h1>
        <div class="stats shadow">
          <div class="stat py-2 px-4">
            <div class="stat-title text-xs">显示器数量</div>
            <div class="stat-value text-primary text-lg">{displayStore.displays.length}</div>
          </div>
        </div>
      </div>

      <LedStripConfigurationContext.Provider value={ledStripConfigurationContextValue}>
        <div class="space-y-4">
          {/* LED Strip Sorter Panel */}
          <div class="card bg-base-200 shadow-lg">
            <div class="card-body p-3">
              <div class="card-title text-sm mb-2">
                <span>灯条排序</span>
                <div class="badge badge-info badge-outline text-xs">实时预览</div>
              </div>
              <LedStripPartsSorter />
              <div class="text-xs text-base-content/50 mt-2">
                💡 提示：拖拽灯条段落来调整顺序，双击可反转方向
              </div>
            </div>
          </div>

          {/* Display Configuration Panel - Auto height based on content */}
          <div class="card bg-base-200 shadow-lg">
            <div class="card-body p-3">
              <div class="card-title text-sm mb-2">
                <span>显示器配置</span>
                <div class="badge badge-secondary badge-outline text-xs">可视化编辑</div>
              </div>
              <div class="mb-3">
                <DisplayListContainer>
                  {displayStore.displays.map((display) => (
                    <DisplayView display={display} />
                  ))}
                </DisplayListContainer>
              </div>
              <div class="text-xs text-base-content/50">
                💡 提示：悬停显示器查看详细信息，使用下方控制面板调整LED数量
              </div>
            </div>
          </div>

          {/* LED Count Control Panels */}
          <div class="flex-shrink-0">
            <div class="flex items-center gap-2 mb-2">
              <h2 class="text-base font-semibold text-base-content">LED数量控制</h2>
              <div class="badge badge-info badge-outline text-xs">实时调整</div>
            </div>
            <div class="led-control-grid">
              {displayStore.displays.map((display) => (
                <LedCountControlPanel display={display} />
              ))}
            </div>
          </div>
        </div>
      </LedStripConfigurationContext.Provider>
    </div>
  );
};
