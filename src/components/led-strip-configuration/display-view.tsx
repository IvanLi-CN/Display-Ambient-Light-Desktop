import { Component, createMemo, createSignal } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { DisplayInfo } from '../../models/display-info.model';
import { displayStore } from '../../stores/display.store';
import { ledStripStore } from '../../stores/led-strip.store';
import { DisplayInfoPanel } from './display-info-panel';
import { LedStripPart } from './led-strip-part';
import { ScreenView } from './screen-view';
import { useLanguage } from '../../i18n/index';
import { WebSocketListener } from '../websocket-listener';
import { LedStripColorsChangedEvent } from '../../types/websocket';


type DisplayViewProps = {
  display: DisplayInfo;
};

export const DisplayView: Component<DisplayViewProps> = (props) => {
  const navigate = useNavigate();
  const { t } = useLanguage();

  // 为当前显示器管理LED颜色数据
  const [displayLedColors, setDisplayLedColors] = createSignal<Map<string, Uint8ClampedArray>>(new Map());

  const size = createMemo(() => ({
    width: props.display.width * displayStore.viewScale,
    height: props.display.height * displayStore.viewScale,
  }));
  const style = createMemo(() => ({
    top: `${props.display.y * displayStore.viewScale}px`,
    left: `${props.display.x * displayStore.viewScale}px`,
    height: `${size().height}px`,
    width: `${size().width}px`,
  }));

  const ledStripConfigs = createMemo(() => {
    // 安全检查：确保 strips 存在且是数组
    if (!ledStripStore.strips || !Array.isArray(ledStripStore.strips)) {
      return [];
    }

    // 使用增强的匹配逻辑，支持V2配置格式
    return ledStripStore.strips.filter((strip) => {
      // 如果strip有matchesDisplay方法，使用它进行匹配
      if (typeof strip.matchesDisplay === 'function') {
        return strip.matchesDisplay(props.display.id, props.display.internal_id);
      }

      // 回退到传统匹配逻辑
      // 优先使用内部ID匹配
      if (strip.display_internal_id && props.display.internal_id) {
        return strip.display_internal_id === props.display.internal_id;
      }

      // 回退到数字ID匹配
      return strip.display_id === props.display.id;
    });
  });

  // 处理LED灯带颜色变化事件
  const handleLedStripColorsChanged = (data: LedStripColorsChangedEvent) => {
    // 只处理属于当前显示器的数据
    if (data.display_id !== props.display.id) {
      return;
    }

    const stripKey = data.border;
    const colors = new Uint8ClampedArray(data.colors);

    setDisplayLedColors(prev => {
      const newMap = new Map(prev);
      newMap.set(stripKey, colors);
      return newMap;
    });
  };

  const handleDisplayClick = () => {
    navigate(`/led-strips-configuration/display/${props.display.id}`);
  };

  return (
    <>
      {/* WebSocket监听器 - 只监听当前显示器的LED灯带颜色变化 */}
      <WebSocketListener
        handlers={{
          onLedStripColorsChanged: handleLedStripColorsChanged,
        }}
        autoConnect={true}
        showStatus={false}
      />

      <section
        class="absolute grid grid-cols-[16px,auto,16px,auto] grid-rows-[16px,auto,16px] overflow-hidden"
        style={style()}
      >
        <ScreenView
          class="row-start-2 col-start-2 group cursor-pointer hover:ring-2 hover:ring-primary hover:ring-opacity-50 transition-all"
          displayId={props.display.id}
          style={{
            'object-fit': 'contain',
          }}
          onClick={handleDisplayClick}
        >
          <DisplayInfoPanel
            display={props.display}
            class="absolute bg-slate-700/20 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded backdrop-blur w-1/3 min-w-[300px] text-black group-hover:opacity-100 opacity-0 transition-opacity pointer-events-none"
          />
          {/* 点击提示 */}
          <div class="absolute bottom-2 right-2 bg-primary/80 text-primary-content px-2 py-1 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
            {t('singleDisplayConfig.virtualDisplayDesc')}
          </div>
        </ScreenView>
        {/* 传递颜色数据给子组件 */}
        <LedStripPart
          class="row-start-1 col-start-2 flex-row overflow-hidden"
          config={ledStripConfigs().find((c) => c.border === 'Top')}
          colors={displayLedColors().get('Top')}
        />
        <LedStripPart
          class="row-start-2 col-start-1 flex-col overflow-hidden"
          config={ledStripConfigs().find((c) => c.border === 'Left')}
          colors={displayLedColors().get('Left')}
        />
        <LedStripPart
          class="row-start-2 col-start-3 flex-col overflow-hidden"
          config={ledStripConfigs().find((c) => c.border === 'Right')}
          colors={displayLedColors().get('Right')}
        />
        <LedStripPart
          class="row-start-3 col-start-2 flex-row overflow-hidden"
          config={ledStripConfigs().find((c) => c.border === 'Bottom')}
          colors={displayLedColors().get('Bottom')}
        />

        {/* LED数量显示在右边 */}
        <div class="row-start-2 col-start-4 flex flex-col justify-center items-start pl-2 text-xs text-base-content/60 space-y-1">
          {ledStripConfigs().map((config) => (
            <div class="flex items-center gap-1">
              <div class={`w-2 h-2 rounded-sm ${
                config.border === 'Top' ? 'bg-blue-400' :
                config.border === 'Right' ? 'bg-red-400' :
                config.border === 'Bottom' ? 'bg-orange-400' :
                'bg-green-400'
              }`} />
              <span>{config.len}</span>
            </div>
          ))}
        </div>
      </section>
    </>
  );
};
