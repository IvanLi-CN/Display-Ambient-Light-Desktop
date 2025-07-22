/**
 * 应用信息相关API服务
 */

import { api } from './api-client';

export class InfoApiService {
  /**
   * 获取应用版本信息
   */
  static async getAppVersion(): Promise<string> {
    const response = await api.get('/api/v1/info/version');
    return (response as any).data;
  }

  /**
   * 报告当前页面信息
   */
  static async reportCurrentPage(pageInfo: string): Promise<void> {
    await api.post('/api/v1/info/current-page', { pageInfo });
  }

  /**
   * 导航到指定页面
   */
  static async navigateToPage(page: string): Promise<void> {
    await api.post('/api/v1/info/navigate', { page });
  }

  /**
   * 导航到显示器配置页面
   */
  static async navigateToDisplayConfig(displayId: string): Promise<void> {
    await api.post('/api/v1/info/navigate-display-config', { displayId });
  }

  /**
   * 打开外部URL
   */
  static async openExternalUrl(url: string): Promise<void> {
    await api.post('/api/v1/info/open-external-url', { url });
  }
}
