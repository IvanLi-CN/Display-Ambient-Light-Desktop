/**
 * LED状态相关的类型定义
 * 与后端 LedStatusStats 保持一致
 */

/**
 * LED数据发送模式
 * 对应后端的 DataSendMode 枚举
 */
export type DataSendMode =
  | 'None'           // 不发送任何数据
  | 'AmbientLight'   // 屏幕氛围光数据
  | 'StripConfig'    // 单灯条配置数据
  | 'TestEffect'     // 测试效果数据
  | 'ColorCalibration'; // 颜色校准数据

/**
 * LED数据发送统计
 * 对应后端的 LedSendStats 结构
 */
export interface LedSendStats {
  /** 总发送包数 */
  total_packets_sent: number;
  /** 总发送字节数 */
  total_bytes_sent: number;
  /** 最后发送时间 */
  last_send_time?: string;
  /** 发送错误次数 */
  send_errors: number;
}

/**
 * LED状态统计信息
 * 对应后端的 LedStatusStats 结构
 */
export interface LedStatusData {
  /** 当前数据发送模式 */
  data_send_mode: DataSendMode;
  /** 测试模式是否激活 */
  test_mode_active: boolean;
  /** 单屏配置模式是否激活 */
  single_display_config_mode: boolean;
  /** 当前活跃的呼吸灯带（display_id, border） */
  active_breathing_strip?: [number, string];
  /** 当前LED颜色数据字节数 */
  current_colors_bytes: number;
  /** 当前排序颜色数据字节数 */
  sorted_colors_bytes: number;
  /** 最后更新时间戳 */
  last_updated: string;
  /** 数据发送统计 */
  send_stats: LedSendStats;
}

/**
 * 状态栏显示用的简化数据结构
 * 从 LedStatusData 提取关键信息用于状态栏显示
 */
export interface StatusBarData {
  /** 当前模式的显示名称 */
  mode: string;
  /** 发送频率（从统计信息计算得出） */
  frequency: number;
  /** 数据长度（字节） */
  data_length: number;
  /** LED总数（从字节数计算得出） */
  total_led_count: number;
  /** 测试模式是否激活 */
  test_mode_active: boolean;
  /** 最后更新时间 */
  last_update: string;
  /** 连接状态 */
  connected: boolean;
}

/**
 * WebSocket状态栏事件数据
 * 对应后端 WsMessage::LedStatusChanged 的数据结构
 */
export interface LedStatusChangedEvent {
  status: LedStatusData;
}

/**
 * 模式显示名称映射
 * 注意：这个映射已被国际化替代，请使用 t('ledStatus.modes.{mode}') 获取翻译
 * @deprecated 使用 t('ledStatus.modes.{mode}') 替代
 */
export const MODE_DISPLAY_NAMES: Record<DataSendMode, string> = {
  'None': '无',
  'AmbientLight': '氛围光',
  'StripConfig': '配置模式',
  'TestEffect': '测试模式',
  'ColorCalibration': '颜色校准'
};

/**
 * 获取国际化的模式显示名称
 */
export const getModeDisplayName = (mode: DataSendMode, t: (key: string) => string): string => {
  return t(`ledStatus.modes.${mode}`);
};

/**
 * 计算发送频率（Hz）
 * 基于发送统计信息计算
 */
export function calculateFrequency(stats: LedSendStats | undefined): number {
  if (!stats || !stats.last_send_time || stats.total_packets_sent === 0) {
    return 0;
  }

  // 这里可以根据实际需要实现更复杂的频率计算逻辑
  // 目前返回一个估算值
  return stats.total_packets_sent > 0 ? 30 : 0; // 默认30Hz
}

/**
 * 计算LED总数
 * 基于颜色数据字节数计算
 */
export function calculateLedCount(colorBytes: number, isRGBW: boolean = false): number {
  const bytesPerLed = isRGBW ? 4 : 3; // RGBW=4字节，RGB=3字节
  return Math.floor(colorBytes / bytesPerLed);
}

/**
 * 格式化时间显示
 */
export function formatTime(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('zh-CN', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  } catch {
    return '--:--:--';
  }
}

/**
 * 将 LedStatusData 转换为 StatusBarData
 */
export function convertToStatusBarData(
  ledStatus: LedStatusData | any,
  connected: boolean = true,
  t?: (key: string) => string
): StatusBarData {
  // 处理可能的数据结构不匹配
  const safeStatus = ledStatus || {};

  const frequency = calculateFrequency(safeStatus.send_stats);
  const totalLedCount = calculateLedCount(safeStatus.current_colors_bytes || 0);

  // 获取模式显示名称
  const mode = (safeStatus.data_send_mode || safeStatus.mode) as DataSendMode;
  const modeDisplayName = t
    ? getModeDisplayName(mode, t)
    : MODE_DISPLAY_NAMES[mode] || '未知';

  return {
    mode: modeDisplayName,
    frequency: safeStatus.frequency || frequency,
    data_length: safeStatus.data_length || safeStatus.current_colors_bytes || 0,
    total_led_count: safeStatus.total_led_count || totalLedCount,
    test_mode_active: safeStatus.test_mode_active || false,
    last_update: formatTime(safeStatus.timestamp || safeStatus.last_updated || new Date().toISOString()),
    connected
  };
}
