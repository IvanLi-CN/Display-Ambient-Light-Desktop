/**
 * API适配器 - 提供Tauri和HTTP API之间的兼容性层
 * 自动检测运行环境并选择合适的API调用方式
 */

// Tauri imports removed - using HTTP API only
import { LedApiService, ConfigApiService } from './led-api.service';
import { DisplayApiService, DeviceApiService, HealthApiService } from './display-api.service';
import { InfoApiService } from './info-api.service';
import { api, WebSocketEventListener } from './api-client';
import { DataSendMode } from '../models/led-data-sender';

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
    console.log('🔧 ApiAdapter.initialize() 被调用');
    if (this.environmentInfo) {
      console.log('🔧 环境信息已存在，直接返回:', this.environmentInfo);
      return this.environmentInfo;
    }

    if (this.initPromise) {
      console.log('🔧 初始化正在进行中，等待完成...');
      await this.initPromise;
      return this.environmentInfo!;
    }

    console.log('🔧 开始初始化环境检测...');
    this.initPromise = this.detectEnvironment();
    await this.initPromise;
    console.log('🔧 环境检测完成:', this.environmentInfo);
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

    // 直接使用 HTTP API
    return await httpApiCall();
  }

  /**
   * 事件监听 - 自动选择Tauri events或WebSocket
   */
  public async onEvent<T>(
    eventName: string,
    handler: (data: T) => void
  ): Promise<() => void> {
    await this.initialize();

    // 只使用WebSocket事件
    return api.onEvent(eventName, (message: any) => {
      handler(message.data || message);
    });
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

  /**
   * 订阅WebSocket事件
   */
  public async subscribeToEvents(eventTypes: string[]): Promise<void> {
    await this.initialize();

    if (this.environmentInfo!.preferredMode === 'http' && this.environmentInfo!.isHttpApiAvailable) {
      // 使用WebSocket订阅 - 发送正确格式的消息
      // 后端期望的格式是 { "Subscribe": { "event_types": [...] } }
      const subscribeMessage = {
        Subscribe: {
          event_types: eventTypes
        }
      };

      // 使用apiClient的sendWebSocketMessage方法
      import('./api-client').then(({ apiClient }) => {
        apiClient.sendWebSocketMessage(subscribeMessage as any);
        console.log('📤 Sent subscription message:', subscribeMessage);
      }).catch(error => {
        console.error('❌ Failed to send subscription message:', error);
      });
    } else {
      console.warn('WebSocket不可用，无法订阅事件');
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

  public async setDataSendMode(mode: DataSendMode): Promise<void> {
    return this.call(
      'set_led_data_send_mode',
      () => LedApiService.setDataSendMode(mode),
      { mode }
    );
  }

  public async getDataSendMode(): Promise<DataSendMode> {
    return this.call(
      'get_led_data_send_mode',
      () => LedApiService.getDataSendMode()
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

  public async getAppVersion(): Promise<any> {
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

  public async patchLedStripLen(displayId: number, border: string, deltaLen: number): Promise<void> {
    return this.call(
      'patch_led_strip_len',
      () => ConfigApiService.patchLedStripLen(displayId, border as any, deltaLen),
      { displayId, border, deltaLen }
    );
  }

  public async patchLedStripType(displayId: number, border: string, ledType: string): Promise<void> {
    return this.call(
      'patch_led_strip_type',
      () => ConfigApiService.patchLedStripType(displayId, border as any, ledType as any),
      { displayId, border, ledType }
    );
  }

  public async moveStripPart(displayId: number, border: string, fromIndex: number, toIndex: number): Promise<void> {
    return this.call(
      'move_strip_part',
      () => ConfigApiService.moveStripPart(displayId, border as any, fromIndex, toIndex),
      { displayId, border, fromIndex, toIndex }
    );
  }

  public async reverseLedStripPart(displayId: number, border: string, startIndex: number, endIndex: number): Promise<void> {
    return this.call(
      'reverse_led_strip_part',
      () => ConfigApiService.reverseLedStripPart(displayId, border as any, startIndex, endIndex),
      { displayId, border, startIndex, endIndex }
    );
  }

  public async setColorCalibration(displayId: number, border: string, calibration: any): Promise<void> {
    return this.call(
      'set_color_calibration',
      () => ConfigApiService.setColorCalibration(displayId, border as any, calibration),
      { displayId, border, calibration }
    );
  }

  public async startSingleDisplayConfigPublisher(strips: any[], borderColors: any): Promise<void> {
    return this.call(
      'start_single_display_config_publisher',
      () => LedApiService.startSingleDisplayConfigPublisher(strips, borderColors),
      { strips, borderColors }
    );
  }

  public async stopSingleDisplayConfigPublisher(): Promise<void> {
    return this.call(
      'stop_single_display_config_publisher',
      () => LedApiService.stopSingleDisplayConfigPublisher()
    );
  }

  public async setActiveStripForBreathing(displayId: number, border: string | null): Promise<void> {
    return this.call(
      'set_active_strip_for_breathing',
      () => LedApiService.setActiveStripForBreathing(displayId, border),
      { displayId, border }
    );
  }

  public async updateUserPreferences(preferences: any): Promise<void> {
    return this.call(
      'update_user_preferences',
      () => ConfigApiService.updateUserPreferences(preferences),
      { preferences }
    );
  }

  public async updateWindowPreferences(windowPrefs: any): Promise<void> {
    return this.call(
      'update_window_preferences',
      () => ConfigApiService.updateWindowPreferences(windowPrefs),
      { windowPrefs }
    );
  }

  public async updateUIPreferences(uiPrefs: any): Promise<void> {
    return this.call(
      'update_ui_preferences',
      () => ConfigApiService.updateUIPreferences(uiPrefs),
      { uiPrefs }
    );
  }

  public async updateViewScale(scale: number): Promise<void> {
    return this.call(
      'update_view_scale',
      () => ConfigApiService.updateViewScale(scale),
      { scale }
    );
  }

  public async updateGlobalColorCalibration(calibration: any): Promise<void> {
    return this.call(
      'set_color_calibration',
      () => ConfigApiService.updateGlobalColorCalibration(calibration),
      { calibration }
    );
  }

  public async updateTheme(theme: string): Promise<void> {
    return this.call(
      'update_theme',
      () => ConfigApiService.updateTheme(theme),
      { theme }
    );
  }

  public async getTheme(): Promise<string> {
    return this.call(
      'get_theme',
      () => ConfigApiService.getTheme()
    );
  }

  public async updateNightModeThemeEnabled(enabled: boolean): Promise<void> {
    return this.call(
      'update_night_mode_theme_enabled',
      () => ConfigApiService.updateNightModeThemeEnabled(enabled),
      { enabled }
    );
  }

  public async updateNightModeTheme(theme: string): Promise<void> {
    return this.call(
      'update_night_mode_theme',
      () => ConfigApiService.updateNightModeTheme(theme),
      { theme }
    );
  }

  public async getUserPreferences(): Promise<any> {
    return this.call(
      'get_user_preferences',
      () => ConfigApiService.getUserPreferences()
    );
  }

  public async getNightModeThemeEnabled(): Promise<boolean> {
    return this.call(
      'get_night_mode_theme_enabled',
      () => ConfigApiService.getNightModeThemeEnabled()
    );
  }

  public async getNightModeTheme(): Promise<string> {
    return this.call(
      'get_night_mode_theme',
      () => ConfigApiService.getNightModeTheme()
    );
  }

  public async getCurrentLanguage(): Promise<string> {
    return this.call(
      'get_current_language',
      () => ConfigApiService.getCurrentLanguage()
    );
  }

  public async setCurrentLanguage(language: string): Promise<void> {
    return this.call(
      'set_current_language',
      () => ConfigApiService.setCurrentLanguage(language),
      { language }
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
  subscribeToEvents: (eventTypes: string[]) => apiAdapter.subscribeToEvents(eventTypes),
  isConnected: () => api.isConnected(),
  
  // LED API
  sendColors: (offset: number, buffer: number[]) => apiAdapter.sendColors(offset, buffer),
  sendTestColorsToBoard: (params: { boardAddress: string, offset: number, buffer: number[] }) =>
    apiAdapter.sendTestColorsToBoard(params.boardAddress, params.offset, params.buffer),
  enableTestMode: () => apiAdapter.enableTestMode(),
  disableTestMode: () => apiAdapter.disableTestMode(),
  setDataSendMode: (mode: DataSendMode) => apiAdapter.setDataSendMode(mode),
  getDataSendMode: () => apiAdapter.getDataSendMode(),
  startLedTestEffect: (params: any) => apiAdapter.startLedTestEffect(params),
  stopLedTestEffect: (params: any) => apiAdapter.stopLedTestEffect(params),
  startSingleDisplayConfigPublisher: (strips: any[], borderColors: any) =>
    apiAdapter.startSingleDisplayConfigPublisher(strips, borderColors),
  stopSingleDisplayConfigPublisher: () => apiAdapter.stopSingleDisplayConfigPublisher(),
  setActiveStripForBreathing: (displayId: number, border: string | null) =>
    apiAdapter.setActiveStripForBreathing(displayId, border),
  
  // 配置API
  readLedStripConfigs: () => apiAdapter.readLedStripConfigs(),
  writeLedStripConfigs: (configs: any) => apiAdapter.writeLedStripConfigs(configs),
  getConfig: () => apiAdapter.getConfig(),
  patchLedStripLen: (displayId: number, border: string, deltaLen: number) =>
    apiAdapter.patchLedStripLen(displayId, border, deltaLen),
  patchLedStripType: (displayId: number, border: string, ledType: string) =>
    apiAdapter.patchLedStripType(displayId, border, ledType),
  moveStripPart: (displayId: number, border: string, fromIndex: number, toIndex: number) =>
    apiAdapter.moveStripPart(displayId, border, fromIndex, toIndex),
  reverseLedStripPart: (displayId: number, border: string, startIndex: number, endIndex: number) =>
    apiAdapter.reverseLedStripPart(displayId, border, startIndex, endIndex),
  setColorCalibration: (displayId: number, border: string, calibration: any) =>
    apiAdapter.setColorCalibration(displayId, border, calibration),
  updateUserPreferences: (preferences: any) => apiAdapter.updateUserPreferences(preferences),
  updateWindowPreferences: (windowPrefs: any) => apiAdapter.updateWindowPreferences(windowPrefs),
  updateUIPreferences: (uiPrefs: any) => apiAdapter.updateUIPreferences(uiPrefs),
  updateViewScale: (scale: number) => apiAdapter.updateViewScale(scale),
  updateGlobalColorCalibration: (calibration: any) => apiAdapter.updateGlobalColorCalibration(calibration),
  updateTheme: (theme: string) => apiAdapter.updateTheme(theme),
  getTheme: () => apiAdapter.getTheme(),
  updateNightModeThemeEnabled: (enabled: boolean) => apiAdapter.updateNightModeThemeEnabled(enabled),
  updateNightModeTheme: (theme: string) => apiAdapter.updateNightModeTheme(theme),
  getUserPreferences: () => apiAdapter.getUserPreferences(),
  getNightModeThemeEnabled: () => apiAdapter.getNightModeThemeEnabled(),
  getNightModeTheme: () => apiAdapter.getNightModeTheme(),
  getCurrentLanguage: () => apiAdapter.getCurrentLanguage(),
  setCurrentLanguage: (language: string) => apiAdapter.setCurrentLanguage(language),
  
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
