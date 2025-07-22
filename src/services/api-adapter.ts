/**
 * API适配器 - 提供Tauri和HTTP API之间的兼容性层
 * 自动检测运行环境并选择合适的API调用方式
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { LedApiService, ConfigApiService } from './led-api.service';
import { DisplayApiService, DeviceApiService, HealthApiService } from './display-api.service';
import { InfoApiService } from './info-api.service';
import { api, WebSocketEventListener } from './api-client';

// 环境检测结果
export interface EnvironmentInfo {
  isTauri: boolean;
  isHttpApiAvailable: boolean;
  preferredMode: 'tauri' | 'http';
}

/**
 * API适配器类
 * 提供统一的API接口，自动选择Tauri或HTTP调用方式
 */
export class ApiAdapter {
  private static instance: ApiAdapter;
  private environmentInfo: EnvironmentInfo | null = null;
  private initPromise: Promise<void> | null = null;

  private constructor() {}

  public static getInstance(): ApiAdapter {
    if (!ApiAdapter.instance) {
      ApiAdapter.instance = new ApiAdapter();
    }
    return ApiAdapter.instance;
  }

  /**
   * 初始化适配器，检测运行环境
   */
  public async initialize(): Promise<EnvironmentInfo> {
    if (this.environmentInfo) {
      return this.environmentInfo;
    }

    if (this.initPromise) {
      await this.initPromise;
      return this.environmentInfo!;
    }

    this.initPromise = this.detectEnvironment();
    await this.initPromise;
    return this.environmentInfo!;
  }

  /**
   * 检测运行环境
   */
  private async detectEnvironment(): Promise<void> {
    // 检测是否在Tauri环境中
    const isTauri = typeof window !== 'undefined' && 
                   !!(window as any).__TAURI__;

    // 检测HTTP API是否可用
    let isHttpApiAvailable = false;
    try {
      isHttpApiAvailable = await HealthApiService.isApiServerAvailable();
    } catch (error) {
      console.warn('HTTP API不可用:', error);
    }

    // 确定首选模式
    let preferredMode: 'tauri' | 'http' = 'tauri';
    if (!isTauri && isHttpApiAvailable) {
      preferredMode = 'http';
    } else if (isHttpApiAvailable) {
      // 如果两者都可用，优先使用HTTP API（更灵活）
      preferredMode = 'http';
    }

    this.environmentInfo = {
      isTauri,
      isHttpApiAvailable,
      preferredMode
    };

    console.log('🔍 环境检测结果:', this.environmentInfo);
  }

  /**
   * 获取环境信息
   */
  public getEnvironmentInfo(): EnvironmentInfo | null {
    return this.environmentInfo;
  }

  /**
   * 通用调用方法 - 自动选择Tauri或HTTP API
   */
  public async call<T>(
    tauriCommand: string,
    httpApiCall: () => Promise<T>,
    tauriArgs?: any
  ): Promise<T> {
    await this.initialize();

    if (this.environmentInfo!.preferredMode === 'http' && this.environmentInfo!.isHttpApiAvailable) {
      try {
        return await httpApiCall();
      } catch (error) {
        console.warn(`HTTP API调用失败，尝试Tauri fallback:`, error);
        if (this.environmentInfo!.isTauri) {
          return await invoke<T>(tauriCommand, tauriArgs);
        }
        throw error;
      }
    } else if (this.environmentInfo!.isTauri) {
      try {
        return await invoke<T>(tauriCommand, tauriArgs);
      } catch (error) {
        console.warn(`Tauri调用失败，尝试HTTP API fallback:`, error);
        if (this.environmentInfo!.isHttpApiAvailable) {
          return await httpApiCall();
        }
        throw error;
      }
    } else {
      throw new Error('没有可用的API调用方式');
    }
  }

  /**
   * 事件监听 - 自动选择Tauri events或WebSocket
   */
  public async onEvent<T>(
    eventName: string,
    handler: (data: T) => void
  ): Promise<() => void> {
    await this.initialize();

    if (this.environmentInfo!.preferredMode === 'http' && this.environmentInfo!.isHttpApiAvailable) {
      // 使用WebSocket事件
      return api.onEvent(eventName, (message) => {
        handler(message.data || message);
      });
    } else if (this.environmentInfo!.isTauri) {
      // 使用Tauri事件
      const unlisten = await listen<T>(eventName, (event) => {
        handler(event.payload);
      });
      return unlisten;
    } else {
      throw new Error('没有可用的事件监听方式');
    }
  }

  /**
   * 发送事件 - 自动选择方式
   */
  public async emitEvent(eventName: string, data: any): Promise<void> {
    await this.initialize();

    if (this.environmentInfo!.preferredMode === 'http' && this.environmentInfo!.isHttpApiAvailable) {
      // 使用WebSocket发送
      api.sendMessage({ type: eventName, data });
    } else if (this.environmentInfo!.isTauri) {
      // Tauri通常不需要从前端发送事件到后端
      console.warn('Tauri环境下不支持从前端发送事件');
    } else {
      throw new Error('没有可用的事件发送方式');
    }
  }

  // ===== LED相关API =====

  public async sendColors(offset: number, buffer: number[]): Promise<void> {
    return this.call(
      'send_colors',
      () => LedApiService.sendColors(offset, buffer),
      { offset, buffer }
    );
  }

  public async sendTestColorsToBoard(boardAddress: string, offset: number, buffer: number[]): Promise<void> {
    return this.call(
      'send_test_colors_to_board',
      () => LedApiService.sendTestColorsToBoard(boardAddress, offset, buffer),
      { boardAddress, offset, buffer }
    );
  }

  public async enableTestMode(): Promise<void> {
    return this.call(
      'enable_test_mode',
      () => LedApiService.enableTestMode()
    );
  }

  public async disableTestMode(): Promise<void> {
    return this.call(
      'disable_test_mode',
      () => LedApiService.disableTestMode()
    );
  }

  // ===== 配置相关API =====

  public async readLedStripConfigs(): Promise<any> {
    return this.call(
      'read_led_strip_configs',
      () => ConfigApiService.readLedStripConfigs()
    );
  }

  public async writeLedStripConfigs(configs: any): Promise<void> {
    return this.call(
      'write_led_strip_configs',
      () => ConfigApiService.writeLedStripConfigs(configs),
      { configs }
    );
  }

  // ===== 显示器相关API =====

  public async listDisplayInfo(): Promise<string> {
    return this.call(
      'list_display_info',
      () => DisplayApiService.listDisplayInfo()
    );
  }

  public async getDisplays(): Promise<any[]> {
    return this.call(
      'get_displays',
      () => DisplayApiService.getDisplays()
    );
  }

  // ===== 设备相关API =====

  public async getBoards(): Promise<any[]> {
    return this.call(
      'get_boards',
      () => DeviceApiService.getBoards()
    );
  }

  // ===== 应用信息相关API =====

  public async getAppVersion(): Promise<string> {
    return this.call(
      'get_app_version_string',
      () => InfoApiService.getAppVersion()
    );
  }

  public async reportCurrentPage(pageInfo: string): Promise<void> {
    return this.call(
      'report_current_page',
      () => InfoApiService.reportCurrentPage(pageInfo),
      { pageInfo }
    );
  }

  public async navigateToPage(page: string): Promise<void> {
    return this.call(
      'navigate_to_page',
      () => InfoApiService.navigateToPage(page),
      { page }
    );
  }

  public async navigateToDisplayConfig(displayId: string): Promise<void> {
    return this.call(
      'navigate_to_display_config',
      () => InfoApiService.navigateToDisplayConfig(displayId),
      { displayId }
    );
  }

  public async openExternalUrl(url: string): Promise<void> {
    return this.call(
      'open_external_url',
      () => InfoApiService.openExternalUrl(url),
      { url }
    );
  }

  public async startLedTestEffect(params: any): Promise<void> {
    return this.call(
      'start_led_test_effect',
      () => LedApiService.startLedTestEffect(params),
      params
    );
  }

  public async stopLedTestEffect(params: any): Promise<void> {
    return this.call(
      'stop_led_test_effect',
      () => LedApiService.stopLedTestEffect(params),
      params
    );
  }

  public async getConfig(): Promise<any> {
    return this.call(
      'read_config',
      () => ConfigApiService.getConfig()
    );
  }

  // ===== 问候API（测试用） =====

  public async greet(name: string): Promise<string> {
    return this.call(
      'greet',
      () => api.post('/api/v1/greet', { name }),
      { name }
    );
  }
}

// 导出默认实例
export const apiAdapter = ApiAdapter.getInstance();

// 导出便捷方法
export const adaptiveApi = {
  // 初始化
  initialize: () => apiAdapter.initialize(),
  getEnvironmentInfo: () => apiAdapter.getEnvironmentInfo(),
  
  // 事件
  onEvent: <T>(eventName: string, handler: (data: T) => void) => apiAdapter.onEvent(eventName, handler),
  emitEvent: (eventName: string, data: any) => apiAdapter.emitEvent(eventName, data),
  
  // LED API
  sendColors: (offset: number, buffer: number[]) => apiAdapter.sendColors(offset, buffer),
  sendTestColorsToBoard: (params: { boardAddress: string, offset: number, buffer: number[] }) =>
    apiAdapter.sendTestColorsToBoard(params.boardAddress, params.offset, params.buffer),
  enableTestMode: () => apiAdapter.enableTestMode(),
  disableTestMode: () => apiAdapter.disableTestMode(),
  startLedTestEffect: (params: any) => apiAdapter.startLedTestEffect(params),
  stopLedTestEffect: (params: any) => apiAdapter.stopLedTestEffect(params),
  
  // 配置API
  readLedStripConfigs: () => apiAdapter.readLedStripConfigs(),
  writeLedStripConfigs: (configs: any) => apiAdapter.writeLedStripConfigs(configs),
  getConfig: () => apiAdapter.getConfig(),
  
  // 显示器API
  listDisplayInfo: () => apiAdapter.listDisplayInfo(),
  getDisplays: () => apiAdapter.getDisplays(),
  
  // 设备API
  getBoards: () => apiAdapter.getBoards(),
  
  // 应用信息API
  getAppVersion: () => apiAdapter.getAppVersion(),
  reportCurrentPage: (pageInfo: string) => apiAdapter.reportCurrentPage(pageInfo),
  navigateToPage: (page: string) => apiAdapter.navigateToPage(page),
  navigateToDisplayConfig: (displayId: string) => apiAdapter.navigateToDisplayConfig(displayId),
  openExternalUrl: (url: string) => apiAdapter.openExternalUrl(url),
  
  // 测试API
  greet: (name: string) => apiAdapter.greet(name),
};
