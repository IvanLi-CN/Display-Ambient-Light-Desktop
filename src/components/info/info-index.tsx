import { Component, For, createEffect, createSignal } from 'solid-js';
import { BoardIndex } from './board-index';
import { WebSocketListener } from '../websocket-listener';
import debug from 'debug';
import { adaptiveApi } from '../../services/api-adapter';
import { DisplayState, RawDisplayState } from '../../models/display-state.model';
import { DisplayStateCard } from '../displays/display-state-card';
import { useLanguage } from '../../i18n/index';

const logger = debug('app:components:info:info-index');

export const InfoIndex: Component = () => {
  const [displayStates, setDisplayStates] = createSignal<DisplayState[]>([]);
  const [displayConfigs, setDisplayConfigs] = createSignal<any[]>([]);
  const { t, locale } = useLanguage();

  createEffect(() => {
    // 获取DDC显示器状态
    adaptiveApi.getDisplays().then((states) => {
      logger('get_displays', states);
      setDisplayStates(
        states.map((it: any) => ({
          ...it,
          last_modified_at: new Date(it.last_modified_at.secs_since_epoch * 1000),
        })),
      );
    });

    // 获取所有显示器配置
    adaptiveApi.getDisplayConfigs().then((configs) => {
      logger('get_display_configs', configs);
      setDisplayConfigs(configs);
    });
  });

  // WebSocket event handlers
  const webSocketHandlers = {
    onDisplaysChanged: (data: any) => {
      logger('displays_changed', data);
      // WebSocket消息格式: { displays: RawDisplayState[] }
      const displays = data.displays || data;
      const displayArray = Array.isArray(displays) ? displays : [];
      setDisplayStates(
        displayArray.map((it: any) => ({
          ...it,
          last_modified_at: new Date(it.last_modified_at.secs_since_epoch * 1000),
        })),
      );
    },
  };

  const getDisplayName = (config: any, index: number) => {
    const rawName = typeof config?.name === 'string' ? config.name.trim() : '';

    if (rawName) {
      if (locale() === 'en-US') {
        if (config?.is_primary && rawName.includes('主显示器')) {
          return t('displays.primaryDisplayName');
        }

        const match = rawName.match(/显示器\s*(\d+)/);
        if (match) {
          return `${t('displays.displayLabel')} ${match[1]}`;
        }
      }

      if (locale() === 'zh-CN') {
        const displayMatch = rawName.match(/Display\s*(\d+)/i);
        if (displayMatch) {
          return `${t('displays.displayLabel')} ${displayMatch[1]}`;
        }
      }

      return rawName;
    }

    if (config?.is_primary) {
      return t('displays.primaryDisplayName');
    }

    return `${t('displays.displayLabel')} ${index + 1}`;
  };

  return (
    <div class="space-y-8">
      <WebSocketListener handlers={webSocketHandlers} />
      {/* 硬件设备信息部分 */}
      <BoardIndex />

      {/* 显示器信息部分 */}
      <div class="space-y-6">
        <div class="flex items-center justify-between">
          <h1 class="text-2xl font-bold text-base-content">{t('displays.title')}</h1>
          <div class="stats shadow">
            <div class="stat">
              <div class="stat-title">{t('displays.displayCount')}</div>
              <div class="stat-value text-primary">{displayConfigs().length}</div>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
          <For each={displayConfigs()}>
            {(config, index) => {
              // 尝试找到对应的DDC状态
              const ddcState = displayStates().find((state, stateIndex) => stateIndex === index());
              const displayName = getDisplayName(config, index());

              return (
                <div class="relative">
                  {ddcState ? (
                    <DisplayStateCard state={ddcState} title={displayName} />
                  ) : (
                    <div class="card bg-base-100 shadow-xl">
                      <div class="card-body">
                        <h2 class="card-title text-base-content">{displayName}</h2>
                        <div class="space-y-2">
                          <div class="flex justify-between items-center py-1">
                            <dt class="text-sm font-medium text-base-content/70">{t('displays.resolution')}</dt>
                            <dd class="text-sm font-mono text-base-content">{config.width}x{config.height}</dd>
                          </div>
                          <div class="flex justify-between items-center py-1">
                            <dt class="text-sm font-medium text-base-content/70">{t('displays.scaleFactor')}</dt>
                            <dd class="text-sm font-mono text-base-content">{config.scale_factor}</dd>
                          </div>
                          <div class="flex justify-between items-center py-1">
                            <dt class="text-sm font-medium text-base-content/70">{t('displays.isPrimary')}</dt>
                            <dd class="text-sm font-mono text-base-content">{config.is_primary ? t('common.yes') : t('common.no')}</dd>
                          </div>
                          <div class="flex justify-between items-center py-1">
                            <dt class="text-sm font-medium text-base-content/70">{t('info.status')}</dt>
                            <dd class="text-sm font-mono text-base-content">
                              <span class="badge badge-warning">{t('displays.ddcUnsupported')}</span>
                            </dd>
                          </div>
                        </div>
                      </div>
                    </div>
                  )}
                  <div class="absolute -top-2 -left-2 w-6 h-6 bg-primary text-primary-content rounded-full flex items-center justify-center text-xs font-bold">
                    {index() + 1}
                  </div>
                </div>
              );
            }}
          </For>
        </div>

        {displayConfigs().length === 0 && (
          <div class="text-center py-12">
            <div class="text-6xl mb-4">🖥️</div>
            <h3 class="text-lg font-semibold text-base-content mb-2">{t('displays.noDisplaysFound')}</h3>
            <p class="text-base-content/70">{t('displays.checkConnection')}</p>
          </div>
        )}
      </div>
    </div>
  );
};
