/**
 * WebSocket消息类型定义
 * 与后端 WsMessage 枚举保持一致
 */

import { DataSendMode } from './led-status';

/**
 * LED颜色变化事件
 */
export interface LedColorsChangedEvent {
  colors: number[];
}

/**
 * LED排序颜色变化事件
 */
export interface LedSortedColorsChangedEvent {
  sorted_colors: number[];
  mode: DataSendMode;
  /** LED偏移量（用于前端组装完整预览） */
  led_offset: number;
  /** 时间戳（来自后端数据生成时间） */
  timestamp?: string;
}

/**
 * LED灯带颜色变化事件（按灯带分组）
 */
export interface LedStripColorsChangedEvent {
  /** 显示器ID */
  display_id: number;
  /** 边框位置 ("Top", "Bottom", "Left", "Right") */
  border: string;
  /** 灯带索引 */
  strip_index: number;
  /** 灯带颜色数据（RGB字节数组） */
  colors: number[];
  /** 数据发送模式 */
  mode: DataSendMode;
}

/**
 * LED状态变化事件
 */
export interface LedStatusChangedEvent {
  status: any; // 使用 any 因为状态结构比较复杂
}

/**
 * 配置变化事件
 */
export interface ConfigChangedEvent {
  config: any;
}

/**
 * 设备列表变化事件
 */
export interface BoardsChangedEvent {
  boards: any;
}

/**
 * 显示器状态变化事件
 */
export interface DisplaysChangedEvent {
  displays: any;
}

/**
 * 环境光状态变化事件
 */
export interface AmbientLightStateChangedEvent {
  state: any;
}

/**
 * LED预览状态变化事件
 */
export interface LedPreviewStateChangedEvent {
  state: any;
}

/**
 * 导航事件
 */
export interface NavigateEvent {
  path: string;
}

/**
 * 订阅事件
 */
export interface SubscribeEvent {
  event_types: string[];
}

/**
 * 取消订阅事件
 */
export interface UnsubscribeEvent {
  event_types: string[];
}

/**
 * 订阅确认事件
 */
export interface SubscriptionConfirmedEvent {
  event_types: string[];
}

/**
 * WebSocket消息联合类型
 */
export type WebSocketMessage = 
  | { type: 'LedColorsChanged'; data: LedColorsChangedEvent }
  | { type: 'LedSortedColorsChanged'; data: LedSortedColorsChangedEvent }
  | { type: 'LedStatusChanged'; data: LedStatusChangedEvent }
  | { type: 'ConfigChanged'; data: ConfigChangedEvent }
  | { type: 'BoardsChanged'; data: BoardsChangedEvent }
  | { type: 'DisplaysChanged'; data: DisplaysChangedEvent }
  | { type: 'AmbientLightStateChanged'; data: AmbientLightStateChangedEvent }
  | { type: 'LedPreviewStateChanged'; data: LedPreviewStateChangedEvent }
  | { type: 'Navigate'; data: NavigateEvent }
  | { type: 'Subscribe'; data: SubscribeEvent }
  | { type: 'Unsubscribe'; data: UnsubscribeEvent }
  | { type: 'SubscriptionConfirmed'; data: SubscriptionConfirmedEvent }
  | { type: 'Ping' }
  | { type: 'Pong' };
