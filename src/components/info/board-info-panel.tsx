import { Component, ParentComponent, createMemo } from 'solid-js';
import { BoardInfo } from '../../models/board-info.model';
import { useLanguage } from '../../i18n/index';

type ItemProps = {
  label: string;
};

const Item: ParentComponent<ItemProps> = (props) => {
  return (
    <div class="flex justify-between items-center py-1">
      <dt class="text-sm font-medium text-base-content/70">{props.label}</dt>
      <dd class="text-sm font-mono text-base-content">{props.children}</dd>
    </div>
  );
};

export const BoardInfoPanel: Component<{ board: BoardInfo }> = (props) => {
  const { t } = useLanguage();
  const ttl = createMemo(() => {
    if (props.board.connect_status !== 'Connected') {
      return '--';
    }

    if (props.board.ttl == null) {
      return t('info.timeout');
    }

    return (
      <>
        <span class="font-mono">{props.board.ttl.toFixed(0)}</span> ms
      </>
    );
  });

  const connectStatus = createMemo(() => {
    if (typeof props.board.connect_status === 'string') {
      // Translate the status string
      switch (props.board.connect_status) {
        case 'Connected':
          return t('info.connected');
        case 'Disconnected':
          return t('info.disconnected');
        case 'Unknown':
          return t('info.unknown');
        default:
          return props.board.connect_status;
      }
    }

    if ('Connecting' in props.board.connect_status) {
      return `${t('info.connecting')} (${props.board.connect_status.Connecting.toFixed(0)})`;
    }
  });

  const statusBadgeClass = createMemo(() => {
    // Use the original status for logic, not the translated text
    const originalStatus = props.board.connect_status;

    if (originalStatus === 'Connected') {
      return 'badge badge-success badge-sm whitespace-nowrap';
    } else if (typeof originalStatus === 'object' && 'Connecting' in originalStatus) {
      return 'badge badge-warning badge-sm whitespace-nowrap';
    } else {
      return 'badge badge-error badge-sm whitespace-nowrap';
    }
  });

  return (
    <div class="card bg-base-200 shadow-lg hover:shadow-xl transition-shadow duration-200">
      <div class="card-body p-4">
        <div class="card-title text-base mb-3 flex items-center justify-between gap-2">
          <span class="truncate flex-1 min-w-0">{props.board.fullname}</span>
          <div class={statusBadgeClass()}>{connectStatus()}</div>
        </div>
        <div class="space-y-2">
          <Item label={t('info.hostname')}>{props.board.host}</Item>
          <Item label={t('info.ipAddress')}>{props.board.address}</Item>
          <Item label={t('info.port')}>{props.board.port}</Item>
          <Item label={t('info.latency')}>{ttl()}</Item>
        </div>
      </div>
    </div>
  );
};
