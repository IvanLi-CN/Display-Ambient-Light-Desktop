import { Component, For, createEffect, createSignal } from 'solid-js';
import { BoardInfo } from '../../models/board-info.model';
import { listen } from '@tauri-apps/api/event';
import debug from 'debug';
import { invoke } from '@tauri-apps/api/core';
import { BoardInfoPanel } from './board-info-panel';

const logger = debug('app:components:info:board-index');

export const BoardIndex: Component = () => {
  const [boards, setBoards] = createSignal<BoardInfo[]>([]);

  createEffect(() => {
    const unlisten = listen<BoardInfo[]>('boards_changed', (ev) => {
      logger('boards_changed', ev);
      setBoards(ev.payload);
    });

    invoke<BoardInfo[]>('get_boards').then((boards) => {
      logger('get_boards', boards);
      setBoards(boards);
    });

    return () => {
      unlisten.then((unlisten) => unlisten());
    };
  });
  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-base-content">设备信息</h1>
        <div class="stats shadow">
          <div class="stat">
            <div class="stat-title">设备总数</div>
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
          <h3 class="text-lg font-semibold text-base-content mb-2">未发现设备</h3>
          <p class="text-base-content/70">请检查设备连接状态</p>
        </div>
      )}
    </div>
  );
};
