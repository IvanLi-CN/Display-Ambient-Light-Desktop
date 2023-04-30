export type BoardInfo = {
  host: string;
  address: string;
  port: number;
  ttl: number;
  connect_status: 'Connected' | 'Disconnected' | { Connecting: number };
  checked_at: Date;
};
