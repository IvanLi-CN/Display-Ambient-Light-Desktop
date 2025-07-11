export type BoardInfo = {
  fullname: string;
  host: string;
  address: string;
  port: number;
  ttl: number;
  connect_status: 'Connected' | 'Disconnected' | 'Unknown' | { Connecting: number };
  checked_at: Date;
};
