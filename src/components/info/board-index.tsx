import { Component, For, createEffect, createSignal } from 'solid-js';
import { BoardInfo } from '../../models/board-info.model';
import { WebSocketListener } from '../websocket-listener';
import debug from 'debug';
import { adaptiveApi } from '../../services/api-adapter';
import { BoardInfoPanel } from './board-info-panel';
import { useLanguage } from '../../i18n/index';

const logger = debug('app:components:info:board-index');

export const BoardIndex: Component = () => {
  const [boards, setBoards] = createSignal<BoardInfo[]>([]);
  const { t } = useLanguage();

  createEffect(() => {
    adaptiveApi.getBoards().then((boards) => {
      logger('get_boards', boards);
      setBoards(boards);
    });
  });

  // WebSocket event handlers
  const webSocketHandlers = {
    boards_changed: (data: BoardInfo[]) => {
      logger('boards_changed', data);
      setBoards(data);
    },
  };
  return (
    <div class="space-y-6">
      <WebSocketListener handlers={webSocketHandlers} />
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-base-content">{t('info.boardInfo')}</h1>
        <div class="stats shadow">
          <div class="stat">
            <div class="stat-title">{t('info.deviceCount')}</div>
            <div class="stat-value text-primary">{boards().length}</div>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <For each={boards()}>
          {(board, index) => (
            <div class="relative">
              <BoardInfoPanel board={board} />
              <div class="absolute -top-2 -left-2 w-6 h-6 bg-primary text-primary-content rounded-full flex items-center justify-center text-xs font-bold">
                {index() + 1}
              </div>
            </div>
          )}
        </For>
      </div>

      {boards().length === 0 && (
        <div class="text-center py-12">
          <div class="text-6xl mb-4">🔍</div>
          <h3 class="text-lg font-semibold text-base-content mb-2">{t('info.noDevicesFound')}</h3>
          <p class="text-base-content/70">{t('info.checkConnection')}</p>
        </div>
      )}
    </div>
  );
};
