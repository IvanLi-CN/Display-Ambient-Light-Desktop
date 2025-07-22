/**
 * 显示器和设备相关API服务
 * 将原有的Tauri invoke调用替换为HTTP API调用
 */

import { api } from './api-client';
import { DisplayInfo } from '../models/display-info.model';
import { DisplayState } from '../models/display-state.model';
import { BoardInfo } from '../models/board-info.model';
import { LedStripConfig } from '../models/led-strip-config';

// LED颜色数据类型
export interface LedColor {
  r: number;
  g: number;
  b: number;
  w?: number;
}

// LED采样点类型
export interface LedSamplePoints {
  border: string;
  points: Array<{ x: number; y: number }>;
}

/**
 * 显示器API服务类
 */
export class DisplayApiService {
  /**
   * 获取所有显示器状态
   * 替代: invoke('get_displays')
   */
  static async getDisplays(): Promise<DisplayState[]> {
    return api.get('/api/v1/display');
  }

  /**
   * 获取显示器信息列表
   * 替代: invoke('list_display_info')
   */
  static async listDisplayInfo(): Promise<string> {
    return api.get('/api/v1/display/info');
  }

  /**
   * 获取指定显示器的颜色
   * 替代: invoke('get_display_colors', { displayId, ledConfigs })
   */
  static async getDisplayColors(
    displayId: number,
    ledConfigs?: LedStripConfig[]
  ): Promise<LedColor[][]> {
    const params: Record<string, any> = {};
    if (ledConfigs) {
      params.led_configs = JSON.stringify(ledConfigs);
    }
    return api.get(`/api/v1/display/${displayId}/colors`, params);
  }

  /**
   * 获取LED灯带采样点
   * 替代: invoke('get_led_strips_sample_points', { displayId })
   */
  static async getLedStripsSamplePoints(displayId: number): Promise<LedSamplePoints[]> {
    return api.get(`/api/v1/display/${displayId}/sample-points`);
  }

  /**
   * 获取单边颜色
   * 替代: invoke('get_one_edge_colors', { displayId, border, ledCount })
   */
  static async getOneEdgeColors(
    displayId: number,
    border: string,
    ledCount: number
  ): Promise<LedColor[]> {
    return api.get(`/api/v1/display/${displayId}/edge-colors`, {
      border,
      led_count: ledCount
    });
  }

  /**
   * 根据LED配置获取颜色
   * 替代: invoke('get_colors_by_led_configs', { displayId, ledConfigs })
   */
  static async getColorsByLedConfigs(
    displayId: number,
    ledConfigs: LedStripConfig[]
  ): Promise<LedColor[][]> {
    return api.post(`/api/v1/display/${displayId}/colors-by-config`, {
      led_configs: ledConfigs
    });
  }
}

/**
 * 设备API服务类
 */
export class DeviceApiService {
  /**
   * 获取设备板列表
   * 替代: invoke('get_boards')
   */
  static async getBoards(): Promise<BoardInfo[]> {
    return api.get('/api/v1/device/boards');
  }

  /**
   * 获取自动启动状态
   * 替代: invoke('get_auto_start_status')
   */
  static async getAutoStartStatus(): Promise<boolean> {
    return api.get('/api/v1/device/auto-start');
  }

  /**
   * 设置自动启动状态
   * 替代: invoke('set_auto_start_status', { enabled })
   */
  static async setAutoStartStatus(enabled: boolean): Promise<void> {
    return api.put('/api/v1/device/auto-start', { enabled });
  }

  /**
   * 获取环境光状态
   * 替代: invoke('get_ambient_light_state')
   */
  static async getAmbientLightState(): Promise<any> {
    return api.get('/api/v1/device/ambient-light-state');
  }
}

/**
 * 应用信息API服务类
 */
export class InfoApiService {
  /**
   * 获取应用版本
   * 替代: invoke('get_app_version') 或 invoke('get_app_version_string')
   */
  static async getAppVersion(): Promise<string> {
    return api.get('/api/v1/info/version');
  }

  /**
   * 获取系统信息
   * 替代: invoke('get_system_info')
   */
  static async getSystemInfo(): Promise<any> {
    return api.get('/api/v1/info/system');
  }

  /**
   * 报告当前页面信息
   * 替代: invoke('report_current_page', { pageInfo })
   */
  static async reportCurrentPage(pageInfo: string): Promise<void> {
    return api.post('/api/v1/info/report-page', { page_info: pageInfo });
  }

  /**
   * 导航到指定页面
   * 替代: invoke('navigate_to_page', { page })
   */
  static async navigateToPage(page: string): Promise<void> {
    return api.post('/api/v1/info/navigate', { page });
  }

  /**
   * 打开外部链接
   * 替代: invoke('open_external_url', { url })
   */
  static async openExternalUrl(url: string): Promise<void> {
    return api.post('/api/v1/info/open-url', { url });
  }
}

/**
 * 健康检查API服务类
 */
export class HealthApiService {
  /**
   * 健康检查
   * 新增功能，检查API服务器状态
   */
  static async healthCheck(): Promise<{ status: string; timestamp: string }> {
    return api.get('/health');
  }

  /**
   * 检查API服务器是否可用
   */
  static async isApiServerAvailable(): Promise<boolean> {
    try {
      await this.healthCheck();
      return true;
    } catch (error) {
      console.warn('API服务器不可用:', error);
      return false;
    }
  }
}

/**
 * 通用问候API（用于测试）
 */
export class GreetApiService {
  /**
   * 问候API
   * 替代: invoke('greet', { name })
   */
  static async greet(name: string): Promise<string> {
    return api.post('/api/v1/greet', { name });
  }
}
