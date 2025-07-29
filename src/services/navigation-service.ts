import { adaptiveApi } from './api-adapter';

/**
 * Navigation service for handling page navigation via Tauri commands
 */
export class NavigationService {
  /**
   * Navigate to a specific page using Tauri command
   * @param page - The page name to navigate to
   * @returns Promise that resolves when navigation is complete
   */
  static async navigateToPage(page: string): Promise<void> {
    try {
      await adaptiveApi.navigateToPage(page);
      console.log(`Successfully navigated to page: ${page}`);
    } catch (error) {
      console.error(`Failed to navigate to page '${page}':`, error);
      throw error;
    }
  }

  /**
   * Navigate to the info page
   */
  static async navigateToInfo(): Promise<void> {
    return this.navigateToPage('info');
  }

  /**
   * Navigate to the LED strips configuration page
   */
  static async navigateToLedConfig(): Promise<void> {
    return this.navigateToPage('led-strips-configuration');
  }

  /**
   * Navigate to the white balance page
   */
  static async navigateToWhiteBalance(): Promise<void> {
    return this.navigateToPage('white-balance');
  }

  /**
   * Navigate to the LED strip test page
   */
  static async navigateToLedTest(): Promise<void> {
    return this.navigateToPage('led-strip-test');
  }



  /**
   * Navigate to the settings page
   */
  static async navigateToSettings(): Promise<void> {
    return this.navigateToPage('settings');
  }

  /**
   * Navigate to a specific display's LED configuration page
   */
  static async navigateToDisplayConfig(displayId: string): Promise<void> {
    try {
      await adaptiveApi.navigateToDisplayConfig(displayId);
      console.log(`Successfully navigated to display config: ${displayId}`);
    } catch (error) {
      console.error(`Failed to navigate to display config '${displayId}':`, error);
      throw error;
    }
  }

  /**
   * Get all available page names
   */
  static getAvailablePages(): string[] {
    return [
      'info',
      'led-strips-configuration',
      'white-balance',
      'led-strip-test',
      'led-data-sender-test',
      'settings'
    ];
  }

  /**
   * Check if a page name is valid
   */
  static isValidPage(page: string): boolean {
    return this.getAvailablePages().includes(page);
  }
}

/**
 * URL scheme helper for creating ambient-light:// URLs
 */
export class AmbientLightUrlScheme {
  /**
   * Create a navigation URL for the ambient-light:// scheme
   * @param page - The page name to navigate to
   * @returns The complete ambient-light:// URL
   */
  static createNavigationUrl(page: string): string {
    if (!NavigationService.isValidPage(page)) {
      throw new Error(`Invalid page name: ${page}`);
    }
    return `ambient-light://navigate/${page}`;
  }

  /**
   * Create navigation URLs for all available pages
   */
  static getAllNavigationUrls(): Record<string, string> {
    const urls: Record<string, string> = {};
    NavigationService.getAvailablePages().forEach(page => {
      urls[page] = this.createNavigationUrl(page);
    });
    return urls;
  }

  /**
   * Open a page using the system's default handler for ambient-light:// URLs
   * This will work if the app is registered as the handler for the URL scheme
   */
  static async openPageViaUrlScheme(page: string): Promise<void> {
    const url = this.createNavigationUrl(page);
    try {
      // Use the browser's location to trigger the URL scheme
      window.location.href = url;
    } catch (error) {
      console.error(`Failed to open URL scheme: ${url}`, error);
      throw error;
    }
  }
}

// Export convenience functions
export const navigateToPage = NavigationService.navigateToPage.bind(NavigationService);
export const navigateToInfo = NavigationService.navigateToInfo.bind(NavigationService);
export const navigateToLedConfig = NavigationService.navigateToLedConfig.bind(NavigationService);
export const navigateToWhiteBalance = NavigationService.navigateToWhiteBalance.bind(NavigationService);
export const navigateToLedTest = NavigationService.navigateToLedTest.bind(NavigationService);
export const navigateToSettings = NavigationService.navigateToSettings.bind(NavigationService);
