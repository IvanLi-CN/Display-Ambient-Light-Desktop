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
    if (props.board.connect_status !== 'Connected') {
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

  const connectStatus = createMemo(() => {
    if (typeof props.board.connect_status === 'string') {
      return props.board.connect_status;
    }

    if ('Connecting' in props.board.connect_status) {
      return `Connecting (${props.board.connect_status.Connecting.toFixed(0)})`;
    }
  });

  return (
    <section class="p-2 rounded shadow">
      <Item label="Host">{props.board.fullname}</Item>
      <Item label="Host">{props.board.host}</Item>
      <Item label="Ip Addr">
        <span class="font-mono">{props.board.address}</span>
      </Item>
      <Item label="Port">
        <span class="font-mono">{props.board.port}</span>
      </Item>
      <Item label="Status">
        <span class="font-mono">{connectStatus()}</span>
      </Item>
      <Item label="TTL">{ttl()}</Item>
    </section>
  );
};
