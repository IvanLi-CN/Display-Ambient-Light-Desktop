/**
 * 全屏工具函数 - 支持浏览器和 Tauri 环境
 * 自动检测运行环境并使用相应的全屏 API
 */

import { getCurrentWindow } from '@tauri-apps/api/window';

/**
 * 检测是否在 Tauri 环境中运行
 */
export function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && !!(window as any).__TAURI__;
}

/**
 * 全屏状态接口
 */
export interface FullscreenState {
  isFullscreen: boolean;
  error?: string;
}

/**
 * 全屏工具类
 */
export class FullscreenManager {
  private static instance: FullscreenManager;
  private currentState: boolean = false;
  private listeners: Set<(state: boolean) => void> = new Set();

  private constructor() {
    this.initializeStateDetection();
  }

  public static getInstance(): FullscreenManager {
    if (!FullscreenManager.instance) {
      FullscreenManager.instance = new FullscreenManager();
    }
    return FullscreenManager.instance;
  }

  /**
   * 初始化状态检测
   */
  private initializeStateDetection(): void {
    if (isTauriEnvironment()) {
      // Tauri 环境：定期检查窗口状态
      this.startTauriStatePolling();
    } else {
      // 浏览器环境：监听 fullscreenchange 事件
      this.initializeBrowserStateDetection();
    }
  }

  /**
   * 浏览器环境状态检测
   */
  private initializeBrowserStateDetection(): void {
    const handleFullscreenChange = () => {
      const newState = !!document.fullscreenElement;
      this.updateState(newState);
    };

    document.addEventListener('fullscreenchange', handleFullscreenChange);
    document.addEventListener('webkitfullscreenchange', handleFullscreenChange);
    document.addEventListener('mozfullscreenchange', handleFullscreenChange);
    document.addEventListener('MSFullscreenChange', handleFullscreenChange);

    // 初始状态检测
    this.updateState(!!document.fullscreenElement);
  }

  /**
   * Tauri 环境状态轮询
   */
  private startTauriStatePolling(): void {
    const checkTauriFullscreenState = async () => {
      try {
        const appWindow = getCurrentWindow();
        const isFullscreen = await appWindow.isFullscreen();
        this.updateState(isFullscreen);
      } catch (error) {
        console.warn('检查 Tauri 全屏状态失败:', error);
      }
    };

    // 初始检查
    checkTauriFullscreenState();

    // 定期检查（每100ms）
    setInterval(checkTauriFullscreenState, 100);
  }

  /**
   * 更新状态并通知监听器
   */
  private updateState(newState: boolean): void {
    if (this.currentState !== newState) {
      this.currentState = newState;
      this.listeners.forEach(listener => listener(newState));
    }
  }

  /**
   * 获取当前全屏状态
   */
  public getCurrentState(): boolean {
    return this.currentState;
  }

  /**
   * 添加状态变化监听器
   */
  public addStateListener(listener: (state: boolean) => void): () => void {
    this.listeners.add(listener);
    // 返回移除监听器的函数
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * 切换全屏状态
   */
  public async toggleFullscreen(): Promise<FullscreenState> {
    try {
      if (isTauriEnvironment()) {
        return await this.toggleTauriFullscreen();
      } else {
        return await this.toggleBrowserFullscreen();
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '未知错误';
      console.warn('全屏切换失败:', errorMessage);
      return {
        isFullscreen: this.currentState,
        error: errorMessage
      };
    }
  }

  /**
   * Tauri 环境全屏切换
   */
  private async toggleTauriFullscreen(): Promise<FullscreenState> {
    const appWindow = getCurrentWindow();
    const currentFullscreen = await appWindow.isFullscreen();
    
    await appWindow.setFullscreen(!currentFullscreen);
    
    // 等待状态更新
    await new Promise(resolve => setTimeout(resolve, 50));
    
    const newState = await appWindow.isFullscreen();
    this.updateState(newState);
    
    console.log(`🖥️ Tauri 全屏状态: ${currentFullscreen} → ${newState}`);
    
    return {
      isFullscreen: newState
    };
  }

  /**
   * 浏览器环境全屏切换
   */
  private async toggleBrowserFullscreen(): Promise<FullscreenState> {
    if (!document.fullscreenElement) {
      // 进入全屏
      await document.documentElement.requestFullscreen();
      console.log('🌐 浏览器进入全屏模式');
    } else {
      // 退出全屏
      await document.exitFullscreen();
      console.log('🌐 浏览器退出全屏模式');
    }

    // 状态会通过事件监听器自动更新
    return {
      isFullscreen: !!document.fullscreenElement
    };
  }

  /**
   * 强制退出全屏
   */
  public async exitFullscreen(): Promise<FullscreenState> {
    try {
      if (isTauriEnvironment()) {
        const appWindow = getCurrentWindow();
        await appWindow.setFullscreen(false);
        const newState = await appWindow.isFullscreen();
        this.updateState(newState);
        return { isFullscreen: newState };
      } else {
        if (document.fullscreenElement) {
          await document.exitFullscreen();
        }
        return { isFullscreen: !!document.fullscreenElement };
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '未知错误';
      console.warn('退出全屏失败:', errorMessage);
      return {
        isFullscreen: this.currentState,
        error: errorMessage
      };
    }
  }
}

/**
 * 全屏管理器单例实例
 */
export const fullscreenManager = FullscreenManager.getInstance();

/**
 * 便捷函数：切换全屏
 */
export const toggleFullscreen = () => fullscreenManager.toggleFullscreen();

/**
 * 便捷函数：退出全屏
 */
export const exitFullscreen = () => fullscreenManager.exitFullscreen();

/**
 * 便捷函数：获取当前状态
 */
export const isFullscreen = () => fullscreenManager.getCurrentState();

/**
 * 便捷函数：添加状态监听器
 */
export const onFullscreenChange = (listener: (state: boolean) => void) => 
  fullscreenManager.addStateListener(listener);
