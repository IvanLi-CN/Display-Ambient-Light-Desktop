/**
 * LED相关API服务
 * 将原有的Tauri invoke调用替换为HTTP API调用
 */

import { api } from './api-client';
import { LedStripConfig, LedType } from '../models/led-strip-config';
import { Borders } from '../constants/border';
import { DataSendMode } from '../models/led-data-sender';

// LED测试效果配置
export interface TestEffectConfig {
  effect_type: string;
  led_count: number;
  led_type: LedType;
  speed: number;
  offset: number;
}

// 边框颜色类型
export interface BorderColors {
  top: number[][];
  bottom: number[][];
  left: number[][];
  right: number[][];
}

// LED状态统计信息
export interface LedStatusStats {
  mode: DataSendMode;
  frequency: number;
  data_length: number;
  total_led_count: number;
  test_mode_active: boolean;
  last_update: string;
}

/**
 * LED API服务类
 */
export class LedApiService {
  /**
   * 发送LED颜色数据
   * 替代: invoke('send_colors', { offset, buffer })
   */
  static async sendColors(offset: number, buffer: number[]): Promise<void> {
    return api.post('/api/v1/led/colors', { offset, buffer });
  }

  /**
   * 发送测试颜色到指定设备
   * 替代: invoke('send_test_colors_to_board', { boardAddress, offset, buffer })
   */
  static async sendTestColorsToBoard(
    boardAddress: string,
    offset: number,
    buffer: number[]
  ): Promise<void> {
    return api.post('/api/v1/led/test-colors', {
      board_address: boardAddress,
      offset,
      buffer
    });
  }

  /**
   * 获取LED数据发送模式
   * 替代: invoke('get_led_data_send_mode')
   */
  static async getDataSendMode(): Promise<DataSendMode> {
    return api.get('/api/v1/led/mode');
  }

  /**
   * 设置LED数据发送模式
   * 替代: invoke('set_led_data_send_mode', { mode })
   */
  static async setDataSendMode(mode: DataSendMode): Promise<void> {
    return api.put('/api/v1/led/mode', { mode });
  }

  /**
   * 获取LED状态统计信息
   */
  static async getLedStatus(): Promise<LedStatusStats> {
    return api.get('/api/v1/led/status');
  }



  /**
   * 启用测试模式
   * 替代: invoke('enable_test_mode')
   */
  static async enableTestMode(): Promise<void> {
    return api.post('/api/v1/led/enable-test-mode');
  }

  /**
   * 禁用测试模式
   * 替代: invoke('disable_test_mode')
   */
  static async disableTestMode(): Promise<void> {
    return api.post('/api/v1/led/disable-test-mode');
  }

  /**
   * 检查测试模式是否激活
   * 替代: invoke('is_test_mode_active')
   */
  static async isTestModeActive(): Promise<boolean> {
    return api.get('/api/v1/led/test-mode-status');
  }





  /**
   * 测试单屏配置模式
   * 替代: invoke('test_single_display_config_mode')
   */
  static async testSingleDisplayConfigMode(): Promise<void> {
    return api.post('/api/v1/led/test-single-display-config');
  }

  /**
   * 测试LED数据发送器
   * 替代: invoke('test_led_data_sender')
   */
  static async testLedDataSender(): Promise<string> {
    return api.post('/api/v1/led/test-data-sender');
  }

  /**
   * 启动LED测试效果
   */
  static async startLedTestEffect(params: any): Promise<void> {
    await api.post('/api/v1/led/start-test-effect', params);
  }

  /**
   * 停止LED测试效果
   */
  static async stopLedTestEffect(params: any): Promise<void> {
    await api.post('/api/v1/led/stop-test-effect', params);
  }

  /**
   * 启动单屏配置发布器
   */
  static async startSingleDisplayConfigPublisher(strips: any[], borderColors: any): Promise<void> {
    await api.post('/api/v1/led/start-single-display-config', { strips, border_colors: borderColors });
  }

  /**
   * 停止单屏配置发布器
   */
  static async stopSingleDisplayConfigPublisher(): Promise<void> {
    await api.post('/api/v1/led/stop-single-display-config');
  }

  /**
   * 设置活跃灯带用于呼吸效果
   */
  static async setActiveStripForBreathing(displayId: number, border: string | null): Promise<void> {
    await api.post('/api/v1/led/set-active-strip-breathing', { display_id: displayId, border });
  }
}

/**
 * 配置相关API服务
 */
export class ConfigApiService {
  /**
   * 获取配置信息
   * 替代: invoke('read_config')
   */
  static async getConfig(): Promise<any> {
    return api.get('/api/v1/config/led-strips');
  }

  /**
   * 读取LED灯带配置
   * 替代: invoke('read_led_strip_configs')
   */
  static async readLedStripConfigs(): Promise<any> {
    return api.get('/api/v1/config/led-strips');
  }

  /**
   * 写入LED灯带配置
   * 替代: invoke('write_led_strip_configs', { configs })
   */
  static async writeLedStripConfigs(configGroup: any): Promise<void> {
    return api.post('/api/v1/config/led-strips', configGroup);
  }

  /**
   * 修改LED灯带长度
   * 替代: invoke('patch_led_strip_len', { displayId, border, deltaLen })
   */
  static async patchLedStripLen(
    displayId: number,
    border: Borders,
    deltaLen: number
  ): Promise<void> {
    return api.put('/api/v1/config/led-strips/length', {
      display_id: displayId,
      border,
      delta_len: deltaLen
    });
  }

  /**
   * 修改LED灯带类型
   * 替代: invoke('patch_led_strip_type', { displayId, border, ledType })
   */
  static async patchLedStripType(
    displayId: number,
    border: Borders,
    ledType: LedType
  ): Promise<void> {
    return api.put('/api/v1/config/led-strips/type', {
      display_id: displayId,
      border,
      led_type: ledType
    });
  }



  /**
   * 更新主题
   * 替代: invoke('update_theme', { theme })
   */
  static async updateTheme(theme: string): Promise<void> {
    return api.put('/api/v1/config/theme', { theme });
  }

  /**
   * 获取主题
   * 替代: invoke('get_theme')
   */
  static async getTheme(): Promise<string> {
    return api.get('/api/v1/config/theme');
  }

  /**
   * 更新用户偏好设置
   * 替代: invoke('update_user_preferences', { preferences })
   */
  static async updateUserPreferences(preferences: any): Promise<void> {
    return api.put('/api/v1/config/user-preferences', preferences);
  }

  /**
   * 更新窗口偏好设置
   * 替代: invoke('update_window_preferences', { windowPrefs })
   */
  static async updateWindowPreferences(windowPrefs: any): Promise<void> {
    return api.put('/api/v1/config/window-preferences', windowPrefs);
  }

  /**
   * 更新UI偏好设置
   * 替代: invoke('update_ui_preferences', { uiPrefs })
   */
  static async updateUIPreferences(uiPrefs: any): Promise<void> {
    return api.put('/api/v1/config/ui-preferences', uiPrefs);
  }

  /**
   * 更新视图缩放
   * 替代: invoke('update_view_scale', { scale })
   */
  static async updateViewScale(scale: number): Promise<void> {
    return api.put('/api/v1/config/view-scale', { scale });
  }

  /**
   * 更新夜间模式主题启用状态
   * 替代: invoke('update_night_mode_theme_enabled', { enabled })
   */
  static async updateNightModeThemeEnabled(enabled: boolean): Promise<void> {
    return api.put('/api/v1/config/night-mode-theme-enabled', { enabled });
  }

  /**
   * 更新夜间模式主题
   * 替代: invoke('update_night_mode_theme', { theme })
   */
  static async updateNightModeTheme(theme: string): Promise<void> {
    return api.put('/api/v1/config/night-mode-theme', { theme });
  }

  /**
   * 更新全局颜色校准
   * 替代: invoke('set_color_calibration', { calibration })
   */
  static async updateGlobalColorCalibration(calibration: any): Promise<void> {
    return api.put('/api/v1/config/global-color-calibration', { calibration });
  }

  /**
   * 获取用户偏好设置
   * 替代: invoke('get_user_preferences')
   */
  static async getUserPreferences(): Promise<any> {
    return api.get('/api/v1/config/user-preferences');
  }

  /**
   * 获取夜间模式主题启用状态
   * 替代: invoke('get_night_mode_theme_enabled')
   */
  static async getNightModeThemeEnabled(): Promise<boolean> {
    return api.get('/api/v1/config/night-mode-theme-enabled');
  }

  /**
   * 获取夜间模式主题
   * 替代: invoke('get_night_mode_theme')
   */
  static async getNightModeTheme(): Promise<string> {
    return api.get('/api/v1/config/night-mode-theme');
  }

  /**
   * 获取当前语言
   * 替代: invoke('get_current_language')
   */
  static async getCurrentLanguage(): Promise<string> {
    return api.get('/api/v1/config/current-language');
  }

  /**
   * 设置当前语言
   * 替代: invoke('set_current_language', { language })
   */
  static async setCurrentLanguage(language: string): Promise<void> {
    return api.put('/api/v1/config/current-language', { language });
  }

  /**
   * 设置颜色校准
   * 替代: invoke('set_color_calibration', { displayId, border, calibration })
   */
  static async setColorCalibration(
    displayId: number,
    border: Borders,
    calibration: any
  ): Promise<void> {
    return api.put('/api/v1/config/color-calibration', {
      display_id: displayId,
      border,
      calibration
    });
  }

  /**
   * 移动灯带部分
   * 替代: invoke('move_strip_part', { displayId, border, fromIndex, toIndex })
   */
  static async moveStripPart(
    displayId: number,
    border: Borders,
    fromIndex: number,
    toIndex: number
  ): Promise<void> {
    return api.put('/api/v1/config/move-strip-part', {
      display_id: displayId,
      border,
      from_index: fromIndex,
      to_index: toIndex
    });
  }

  /**
   * 反转LED灯带部分
   * 替代: invoke('reverse_led_strip_part', { displayId, border, startIndex, endIndex })
   */
  static async reverseLedStripPart(
    displayId: number,
    border: Borders,
    startIndex: number,
    endIndex: number
  ): Promise<void> {
    return api.put('/api/v1/config/reverse-strip-part', {
      display_id: displayId,
      border,
      start_index: startIndex,
      end_index: endIndex
    });
  }

}
