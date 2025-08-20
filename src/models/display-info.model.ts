export class DisplayInfo {
  constructor(
    public id: number,
    public x: number,
    public y: number,
    public width: number,
    public height: number,
    public scale_factor: number,
    public is_primary: boolean,
    /**
     * V2配置格式支持：显示器内部ID
     * 用于稳定的显示器标识，不受系统重启影响
     */
    public internal_id?: string,
    /** 显示器自定义名称 */
    public name?: string,
  ) {}

  /**
   * 获取显示器的标识符
   * 优先返回内部ID，回退到数字ID
   */
  getIdentifier(): string | number {
    return this.internal_id || this.id;
  }

  /**
   * 获取显示器的显示名称
   * 优先使用自定义名称，回退到默认格式
   */
  getDisplayName(): string {
    if (this.name) {
      return this.name;
    }
    return this.is_primary ? '主显示器' : `显示器 ${this.id}`;
  }
}