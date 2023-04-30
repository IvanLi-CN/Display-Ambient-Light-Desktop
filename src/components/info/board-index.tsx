import { Component, For, createEffect, createSignal } from 'solid-js';
import { BoardInfo } from '../../models/board-info.model';
import { listen } from '@tauri-apps/api/event';
import debug from 'debug';
import { invoke } from '@tauri-apps/api';
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
    <ol class="grid sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 p-2 gap-2">
      <For each={boards()}>
        {(board, index) => (
          <li class="bg-slate-50 text-gray-800 relative border-2 border-slate-50 hover:border-sky-300 focus:border-sky-300 transition">
            <BoardInfoPanel board={board} />
            <span class="absolute left-2 -top-3 bg-sky-300 text-white px-1 py-0.5 text-xs rounded-sm font-mono">
              #{index() + 1}
            </span>
          </li>
        )}
      </For>
    </ol>
  );
};
