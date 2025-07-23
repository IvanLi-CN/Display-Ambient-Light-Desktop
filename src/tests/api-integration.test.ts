/**
 * API集成测试套件
 * 验证前端API调用与后端实现的一致性
 */

import { api } from '../services/api-client';
import { LedApiService, ConfigApiService } from '../services/led-api.service';
import { DisplayApiService, DeviceApiService, InfoApiService } from '../services/display-api.service';
import { HealthApiService } from '../services/display-api.service';

interface TestResult {
  name: string;
  endpoint: string;
  method: string;
  status: 'success' | 'error' | 'warning';
  message: string;
  responseTime?: number;
  statusCode?: number;
}

class ApiIntegrationTester {
  private results: TestResult[] = [];

  /**
   * 运行单个测试
   */
  private async runTest(
    name: string,
    endpoint: string,
    method: string,
    testFn: () => Promise<any>
  ): Promise<TestResult> {
    const startTime = Date.now();
    
    try {
      await testFn();
      const responseTime = Date.now() - startTime;
      
      const result: TestResult = {
        name,
        endpoint,
        method,
        status: 'success',
        message: '测试通过',
        responseTime,
        statusCode: 200
      };
      
      this.results.push(result);
      console.log(`✅ ${name}: ${result.message} (${responseTime}ms)`);
      return result;
      
    } catch (error: any) {
      const responseTime = Date.now() - startTime;
      const statusCode = error.message?.match(/HTTP (\d+)/)?.[1];
      
      const result: TestResult = {
        name,
        endpoint,
        method,
        status: 'error',
        message: error.message || '未知错误',
        responseTime,
        statusCode: statusCode ? parseInt(statusCode) : undefined
      };
      
      this.results.push(result);
      console.error(`❌ ${name}: ${result.message} (${responseTime}ms)`);
      return result;
    }
  }

  /**
   * 测试健康检查API
   */
  async testHealthCheck() {
    await this.runTest(
      '健康检查',
      '/health',
      'GET',
      () => HealthApiService.healthCheck()
    );
  }

  /**
   * 测试通用API
   */
  async testGeneralApis() {
    // 问候API
    await this.runTest(
      '问候API',
      '/api/v1/greet',
      'POST',
      () => api.post('/api/v1/greet', { name: 'Test User' })
    );

    // Ping API
    await this.runTest(
      'Ping API',
      '/api/v1/ping',
      'GET',
      () => api.get('/api/v1/ping')
    );
  }

  /**
   * 测试信息API
   */
  async testInfoApis() {
    // 应用版本
    await this.runTest(
      '获取应用版本',
      '/api/v1/info/version',
      'GET',
      () => InfoApiService.getAppVersion()
    );

    // 系统信息
    await this.runTest(
      '获取系统信息',
      '/api/v1/info/system',
      'GET',
      () => api.get('/api/v1/info/system')
    );

    // 报告当前页面 - 测试两个不同的端点
    await this.runTest(
      '报告当前页面 (current-page)',
      '/api/v1/info/current-page',
      'POST',
      () => api.post('/api/v1/info/current-page', { page_info: 'test-page' })
    );

    await this.runTest(
      '报告当前页面 (report-page)',
      '/api/v1/info/report-page',
      'POST',
      () => api.post('/api/v1/info/report-page', { page_info: 'test-page' })
    );

    // 导航
    await this.runTest(
      '导航到页面',
      '/api/v1/info/navigate',
      'POST',
      () => api.post('/api/v1/info/navigate', { page: 'test-page' })
    );

    await this.runTest(
      '导航到显示器配置',
      '/api/v1/info/navigate-display-config',
      'POST',
      () => api.post('/api/v1/info/navigate-display-config', { display_id: '1' })
    );
  }

  /**
   * 测试配置API
   */
  async testConfigApis() {
    // LED配置
    await this.runTest(
      '获取LED配置',
      '/api/v1/config/led-strips',
      'GET',
      () => ConfigApiService.readLedStripConfigs()
    );

    // 用户偏好设置
    await this.runTest(
      '获取用户偏好设置',
      '/api/v1/config/user-preferences',
      'GET',
      () => ConfigApiService.getUserPreferences()
    );

    // 主题
    await this.runTest(
      '获取主题',
      '/api/v1/config/theme',
      'GET',
      () => ConfigApiService.getTheme()
    );

    // 夜间模式主题
    await this.runTest(
      '获取夜间模式主题启用状态',
      '/api/v1/config/night-mode-theme-enabled',
      'GET',
      () => ConfigApiService.getNightModeThemeEnabled()
    );

    await this.runTest(
      '获取夜间模式主题',
      '/api/v1/config/night-mode-theme',
      'GET',
      () => ConfigApiService.getNightModeTheme()
    );

    // 视图缩放
    await this.runTest(
      '获取视图缩放',
      '/api/v1/config/view-scale',
      'GET',
      () => api.get('/api/v1/config/view-scale')
    );

    // 当前语言
    await this.runTest(
      '获取当前语言',
      '/api/v1/config/current-language',
      'GET',
      () => ConfigApiService.getCurrentLanguage()
    );
  }

  /**
   * 测试LED API
   */
  async testLedApis() {
    // 数据发送模式
    await this.runTest(
      '获取LED数据发送模式',
      '/api/v1/led/mode',
      'GET',
      () => LedApiService.getDataSendMode()
    );

    // 测试模式状态
    await this.runTest(
      '获取测试模式状态',
      '/api/v1/led/test-mode-status',
      'GET',
      () => LedApiService.isTestModeActive()
    );
  }

  /**
   * 测试显示器API
   */
  async testDisplayApis() {
    // 显示器列表
    await this.runTest(
      '获取显示器列表',
      '/api/v1/display',
      'GET',
      () => DisplayApiService.getDisplays()
    );

    // 显示器信息
    await this.runTest(
      '获取显示器信息',
      '/api/v1/display/info',
      'GET',
      () => DisplayApiService.listDisplayInfo()
    );
  }

  /**
   * 测试设备API
   */
  async testDeviceApis() {
    // 设备列表
    await this.runTest(
      '获取设备列表',
      '/api/v1/device/boards',
      'GET',
      () => DeviceApiService.getBoards()
    );

    // 自动启动状态
    await this.runTest(
      '获取自动启动状态',
      '/api/v1/device/auto-start',
      'GET',
      () => DeviceApiService.getAutoStartStatus()
    );

    // 环境光状态
    await this.runTest(
      '获取环境光状态',
      '/api/v1/device/ambient-light-state',
      'GET',
      () => DeviceApiService.getAmbientLightState()
    );
  }

  /**
   * 运行所有测试
   */
  async runAllTests(): Promise<TestResult[]> {
    console.log('🚀 开始API集成测试...\n');
    
    this.results = [];
    
    await this.testHealthCheck();
    await this.testGeneralApis();
    await this.testInfoApis();
    await this.testConfigApis();
    await this.testLedApis();
    await this.testDisplayApis();
    await this.testDeviceApis();
    
    this.printSummary();
    return this.results;
  }

  /**
   * 打印测试摘要
   */
  private printSummary() {
    const total = this.results.length;
    const success = this.results.filter(r => r.status === 'success').length;
    const errors = this.results.filter(r => r.status === 'error').length;
    const warnings = this.results.filter(r => r.status === 'warning').length;
    
    console.log('\n📊 测试摘要:');
    console.log(`总计: ${total}`);
    console.log(`✅ 成功: ${success}`);
    console.log(`❌ 失败: ${errors}`);
    console.log(`⚠️  警告: ${warnings}`);
    console.log(`成功率: ${((success / total) * 100).toFixed(1)}%`);
    
    if (errors > 0) {
      console.log('\n❌ 失败的测试:');
      this.results
        .filter(r => r.status === 'error')
        .forEach(r => {
          console.log(`  - ${r.name}: ${r.message} (${r.method} ${r.endpoint})`);
        });
    }
  }

  /**
   * 获取测试结果
   */
  getResults(): TestResult[] {
    return this.results;
  }
}

// 导出测试器实例
export const apiTester = new ApiIntegrationTester();

// 导出测试结果类型
export type { TestResult };
