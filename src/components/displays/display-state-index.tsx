import { Component, For, createEffect, createSignal } from 'solid-js';
import { listen } from '@tauri-apps/api/event';
import debug from 'debug';
import { invoke } from '@tauri-apps/api/core';
import { DisplayState, RawDisplayState } from '../../models/display-state.model';
import { DisplayStateCard } from './display-state-card';

const logger = debug('app:components:displays:display-state-index');

export const DisplayStateIndex: Component = () => {
  const [states, setStates] = createSignal<DisplayState[]>([]);

  createEffect(() => {
    const unlisten = listen<RawDisplayState[]>('displays_changed', (ev) => {
      logger('displays_changed', ev);
      setStates(
        ev.payload.map((it) => ({
          ...it,
          last_modified_at: new Date(it.last_modified_at.secs_since_epoch * 1000),
        })),
      );
    });

    invoke<RawDisplayState[]>('get_displays').then((states) => {
      logger('get_displays', states);
      setStates(
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
    <ol class="grid sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 p-2 gap-2">
      <For each={states()}>
        {(state, index) => (
          <li class="bg-slate-50 text-gray-800 relative border-2 border-slate-50 hover:border-sky-300 focus:border-sky-300 transition">
            <DisplayStateCard state={state} />
            <span class="absolute left-2 -top-3 bg-sky-300 text-white px-1 py-0.5 text-xs rounded-sm font-mono">
              #{index() + 1}
            </span>
          </li>
        )}
      </For>
    </ol>
  );
};
