import { Component, ParentComponent, createMemo } from 'solid-js';
import { BoardInfo } from '../../models/board-info.model';

type ItemProps = {
  label: string;
};

const Item: ParentComponent<ItemProps> = (props) => {
  return (
    <dl class="flex">
      <dt class="w-20">{props.label}</dt>
      <dd class="flex-auto">{props.children}</dd>
    </dl>
  );
};

export const BoardInfoPanel: Component<{ board: BoardInfo }> = (props) => {
  const ttl = createMemo(() => {
    if (!props.board.is_online) {
      return '--';
    }

    if (props.board.ttl == null) {
      return 'timeout';
    }

    return (
      <>
        <span class="font-mono">{props.board.ttl.toFixed(0)}</span> ms
      </>
    );
  });

  return (
    <section class="p-2 rounded shadow">
      <Item label="Host">{props.board.host}</Item>
      <Item label="Ip Addr">
        <span class="font-mono">{props.board.address}</span>
      </Item>
      <Item label="Port">
        <span class="font-mono">{props.board.port}</span>
      </Item>
      <Item label="Status">
        <span class="font-mono">{props.board.is_online ? 'Online' : 'Offline'}</span>
      </Item>
      <Item label="TTL">{ttl()}</Item>
    </section>
  );
};
