/**
 * LED数据发送模式
 */
export enum DataSendMode {
  /** 不发送任何数据 */
  None = 'None',
  /** 屏幕氛围光数据 */
  AmbientLight = 'AmbientLight',
  /** 单灯条配置数据 */
  StripConfig = 'StripConfig',
  /** 测试效果数据 */
  TestEffect = 'TestEffect',
}

/**
 * LED数据发送器相关的API接口
 */
export interface LedDataSenderAPI {
  /** 获取当前数据发送模式 */
  getLedDataSendMode(): Promise<DataSendMode>;
  
  /** 设置数据发送模式 */
  setLedDataSendMode(mode: DataSendMode): Promise<void>;
  
  /** 测试LED数据发送器 */
  testLedDataSender(): Promise<string>;
}
