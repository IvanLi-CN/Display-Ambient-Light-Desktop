/**
 * 颜色校准服务
 * 负责处理颜色校准界面的颜色解析、LED数据生成和硬件发送
 */

import { adaptiveApi } from './api-adapter';
import { ledStripStore } from '../stores/led-strip.store';
import { LedStripConfig, LedType } from '../models/led-strip-config';
import { DataSendMode } from '../types/led-status';

/**
 * RGB颜色数据
 */
export interface RgbColor {
  r: number;
  g: number;
  b: number;
}

/**
 * 颜色校准服务类
 */
export class ColorCalibrationService {
  private static instance: ColorCalibrationService;
  private previousMode: DataSendMode | null = null;
  private isActive = false;

  private constructor() {}

  public static getInstance(): ColorCalibrationService {
    if (!ColorCalibrationService.instance) {
      ColorCalibrationService.instance = new ColorCalibrationService();
    }
    return ColorCalibrationService.instance;
  }

  /**
   * 启用颜色校准模式
   */
  public async enableColorCalibrationMode(): Promise<void> {
    if (this.isActive) {
      return;
    }

    try {
      // 保存当前模式
      this.previousMode = await adaptiveApi.getDataSendMode();

      // 切换到颜色校准模式
      await adaptiveApi.setDataSendMode('ColorCalibration');

      this.isActive = true;
      console.log('✅ 颜色校准模式已启用');
    } catch (error) {
      console.error('❌ 启用颜色校准模式失败:', error);
      throw error;
    }
  }

  /**
   * 禁用颜色校准模式
   */
  public async disableColorCalibrationMode(): Promise<void> {
    if (!this.isActive) {
      return;
    }

    try {
      // 恢复之前的模式
      if (this.previousMode !== null) {
        await adaptiveApi.setDataSendMode(this.previousMode);
      } else {
        await adaptiveApi.setDataSendMode('None');
      }

      this.isActive = false;
      this.previousMode = null;
      console.log('✅ 颜色校准模式已禁用');
    } catch (error) {
      console.error('❌ 禁用颜色校准模式失败:', error);
      throw error;
    }
  }

  /**
   * 解析hex颜色为RGB
   */
  public parseHexColor(hexColor: string): RgbColor {
    // 移除#号
    const hex = hexColor.replace('#', '');
    
    // 解析RGB值
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    
    return { r, g, b };
  }

  /**
   * 生成所有LED的颜色数据
   */
  public generateLedColorData(color: RgbColor): Uint8Array {
    const strips = ledStripStore.strips;
    const colorCalibration = ledStripStore.colorCalibration;
    
    if (!strips || strips.length === 0) {
      console.warn('⚠️ 没有找到LED灯带配置');
      return new Uint8Array(0);
    }

    // 应用颜色校准
    const calibratedColor = {
      r: Math.round(color.r * colorCalibration.r),
      g: Math.round(color.g * colorCalibration.g),
      b: Math.round(color.b * colorCalibration.b),
      w: Math.round(255 * colorCalibration.w), // 白色通道
    };

    // 按序列号排序灯带
    const sortedStrips = [...strips].sort((a, b) => {
      // 检查是否有index属性（后端数据结构）
      if ('index' in a && 'index' in b) {
        return (a as any).index - (b as any).index;
      }
      // 前端数据结构的备用排序方式：先按display_id，再按border
      if (a.display_id !== b.display_id) {
        return a.display_id - b.display_id;
      }
      // 按边框顺序排序：Top, Right, Bottom, Left
      const borderOrder = { 'Top': 0, 'Right': 1, 'Bottom': 2, 'Left': 3 };
      const aOrder = borderOrder[a.border as keyof typeof borderOrder] ?? 999;
      const bOrder = borderOrder[b.border as keyof typeof borderOrder] ?? 999;
      return aOrder - bOrder;
    });

    // 计算总字节数
    let totalBytes = 0;
    for (const strip of sortedStrips) {
      const bytesPerLed = strip.led_type === LedType.SK6812 ? 4 : 3;
      totalBytes += strip.len * bytesPerLed;
    }

    const buffer = new Uint8Array(totalBytes);
    let bufferOffset = 0;

    // 为每个灯带生成数据
    for (const strip of sortedStrips) {
      const bytesPerLed = strip.led_type === LedType.SK6812 ? 4 : 3;
      
      for (let i = 0; i < strip.len; i++) {
        if (strip.led_type === LedType.SK6812) {
          // SK6812: G,R,B,W 顺序
          buffer[bufferOffset] = calibratedColor.g;
          buffer[bufferOffset + 1] = calibratedColor.r;
          buffer[bufferOffset + 2] = calibratedColor.b;
          buffer[bufferOffset + 3] = calibratedColor.w;
        } else {
          // WS2812B: G,R,B 顺序
          buffer[bufferOffset] = calibratedColor.g;
          buffer[bufferOffset + 1] = calibratedColor.r;
          buffer[bufferOffset + 2] = calibratedColor.b;
        }
        bufferOffset += bytesPerLed;
      }
    }

    return buffer;
  }

  /**
   * 应用颜色到所有LED
   */
  public async applyColorToAllLeds(hexColor: string): Promise<void> {
    if (!this.isActive) {
      console.warn('⚠️ 颜色校准模式未启用');
      return;
    }

    try {
      // 解析颜色
      const rgbColor = this.parseHexColor(hexColor);
      console.log('🎨 解析颜色:', hexColor, '→', rgbColor);

      // 生成LED数据
      const ledData = this.generateLedColorData(rgbColor);
      console.log('📦 生成LED数据:', ledData.length, '字节');

      if (ledData.length === 0) {
        console.warn('⚠️ 没有生成LED数据');
        return;
      }

      // 发送到硬件 (使用偏移量0，发送完整数据流)
      await adaptiveApi.sendColors(0, Array.from(ledData));
      console.log('✅ 颜色已应用到所有LED');
    } catch (error) {
      console.error('❌ 应用颜色到LED失败:', error);
      throw error;
    }
  }

  /**
   * 清除所有LED (设置为黑色)
   */
  public async clearAllLeds(): Promise<void> {
    await this.applyColorToAllLeds('#000000');
  }

  /**
   * 获取当前状态
   */
  public getStatus() {
    return {
      isActive: this.isActive,
      previousMode: this.previousMode,
      totalLedCount: ledStripStore.totalLedCount,
      stripCount: ledStripStore.strips.length,
    };
  }
}

// 导出单例实例
export const colorCalibrationService = ColorCalibrationService.getInstance();
