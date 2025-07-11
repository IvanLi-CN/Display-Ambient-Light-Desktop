import { Component, For, createEffect, createSignal } from 'solid-js';
import { BoardIndex } from './board-index';
import { listen } from '@tauri-apps/api/event';
import debug from 'debug';
import { invoke } from '@tauri-apps/api/core';
import { DisplayState, RawDisplayState } from '../../models/display-state.model';
import { DisplayStateCard } from '../displays/display-state-card';
import { useLanguage } from '../../i18n/index';

const logger = debug('app:components:info:info-index');

export const InfoIndex: Component = () => {
  const [displayStates, setDisplayStates] = createSignal<DisplayState[]>([]);
  const { t } = useLanguage();

  createEffect(() => {
    const unlisten = listen<RawDisplayState[]>('displays_changed', (ev) => {
      logger('displays_changed', ev);
      setDisplayStates(
        ev.payload.map((it) => ({
          ...it,
          last_modified_at: new Date(it.last_modified_at.secs_since_epoch * 1000),
        })),
      );
    });

    invoke<RawDisplayState[]>('get_displays').then((states) => {
      logger('get_displays', states);
      setDisplayStates(
        states.map((it) => ({
          ...it,
          last_modified_at: new Date(it.last_modified_at.secs_since_epoch * 1000),
        })),
      );
    });

    return () => {
      unlisten.then((unlisten) => unlisten());
    };
  });

  return (
    <div class="space-y-8">
      {/* 硬件设备信息部分 */}
      <BoardIndex />

      {/* 显示器信息部分 */}
      <div class="space-y-6">
        <div class="flex items-center justify-between">
          <h1 class="text-2xl font-bold text-base-content">{t('displays.title')}</h1>
          <div class="stats shadow">
            <div class="stat">
              <div class="stat-title">{t('displays.displayCount')}</div>
              <div class="stat-value text-primary">{displayStates().length}</div>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
          <For each={displayStates()}>
            {(state, index) => (
              <div class="relative">
                <DisplayStateCard state={state} />
                <div class="absolute -top-2 -left-2 w-6 h-6 bg-primary text-primary-content rounded-full flex items-center justify-center text-xs font-bold">
                  {index() + 1}
                </div>
              </div>
            )}
          </For>
        </div>

        {displayStates().length === 0 && (
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
