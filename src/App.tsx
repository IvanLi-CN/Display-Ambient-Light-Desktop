import { Route, useLocation, useNavigate, A, Navigate } from '@solidjs/router';
import { LedStripConfiguration } from './components/led-strip-configuration/led-strip-configuration';
import { SingleDisplayConfig } from './components/led-strip-configuration/single-display-config';
import { WhiteBalance } from './components/white-balance/white-balance';
import { LedStripTest } from './components/led-strip-test/led-strip-test';
import { Settings } from './components/settings/settings';
import { StatusBar } from './components/status-bar/status-bar';
import { createEffect, createSignal, onMount } from 'solid-js';
import { adaptiveApi } from './services/api-adapter';
import { setLedStripStore } from './stores/led-strip.store';
import { LedStripConfigContainer } from './models/led-strip-config';
import { InfoIndex } from './components/info/info-index';
import { useLanguage } from './i18n/index';
import { AppVersion } from './models/app-version.model';
// Import theme store and initialize effects
import { initializeThemeEffects } from './stores/theme.store';
// Import user preferences store to ensure it's initialized
import { userPreferencesStore } from './stores/user-preferences.store';

function App(props: { children?: any }) {
  const location = useLocation();
  const navigate = useNavigate();
  const [previousPath, setPreviousPath] = createSignal<string>('');
  const { t } = useLanguage();
  const [appVersion, setAppVersion] = createSignal<AppVersion>({ version: '1.0.0', is_dev: false });

  // Get app version on mount
  createEffect(() => {
    adaptiveApi.getAppVersion().then((version: any) => {
      setAppVersion(version);
    }).catch((error: any) => {
      console.error('Failed to get app version:', error);
    });
  });

  // Initialize user preferences on mount
  createEffect(() => {
    userPreferencesStore.initializePreferences().catch((error) => {
      console.error('Failed to initialize user preferences:', error);
    });
  });

  // Initialize theme effects
  onMount(() => {
    initializeThemeEffects();
  });

  // Listen for navigation events from backend
  onMount(async () => {
    try {
      // 初始化API适配器
      await adaptiveApi.initialize();

      const unlisten = await adaptiveApi.onEvent<string>('navigate', (targetPath) => {
        // 只在开发模式下记录导航日志
        if (import.meta.env.DEV) {
          console.log('Navigation event:', targetPath);
        }

        // Use SolidJS navigate function for proper routing
        navigate(targetPath);

        // Report successful navigation
        adaptiveApi.reportCurrentPage(`Navigation event processed: ${targetPath}`)
          .catch((error) => {
            console.error('Failed to report navigation event:', error);
          });
      });

      console.log('✅ Navigation event listener registered');

      // Cleanup function will be called when component unmounts
      return unlisten;
    } catch (error) {
      console.error('❌ Failed to register navigation event listener:', error);
    }
  });



  // Monitor route changes and reset LED mode to AmbientLight before entering any page
  createEffect(() => {
    const currentPath = location.pathname;
    const prevPath = previousPath();

    // Report current page to backend for verification
    const getPageName = (path: string) => {
      if (path === '/info') return 'info';
      if (path === '/led-strips-configuration') return 'led-strips-configuration';
      if (path.startsWith('/led-strips-configuration/display/')) {
        const displayId = path.split('/').pop();
        return `led-strips-configuration/display/${displayId}`;
      }
      if (path === '/color-calibration') return 'color-calibration';
      if (path === '/led-strip-test') return 'led-strip-test';
      if (path === '/led-data-sender-test') return 'led-data-sender-test';
      if (path === '/settings') return 'settings';
      return path;
    };

    const currentPageName = getPageName(currentPath);
    adaptiveApi.reportCurrentPage(`Current page: ${currentPageName} (path: ${currentPath})`).catch((error) => {
      console.error('Failed to report current page:', error);
    });

    // Reset LED mode to AmbientLight before entering any page (except on initial load and LED test pages)
    if (prevPath !== '' && !currentPath.includes('/led-strip-test')) {
      // 只在开发模式下记录路由变化日志
      if (import.meta.env.DEV) {
        console.log(`Route change: ${prevPath} -> ${currentPath}, resetting LED mode`);
      }
      adaptiveApi.setDataSendMode('AmbientLight').then(() => {
        if (import.meta.env.DEV) {
          console.log('LED mode reset to AmbientLight');
        }
      }).catch((error) => {
        console.error('Failed to reset LED mode to AmbientLight:', error);
      });
    }

    // Update previousPath after the condition check
    setPreviousPath(currentPath);
  });

  createEffect(() => {
    adaptiveApi.getConfig().then((config: LedStripConfigContainer) => {
      console.log('🔧 App.tsx - 获取到的配置数据:', config);

      // 安全检查：确保 strips 存在且是数组
      if (config && config.strips && Array.isArray(config.strips)) {
        console.log('✅ App.tsx - 有效的配置数据，strips数量:', config.strips.length);
        setLedStripStore({
          strips: config.strips,
          colorCalibration: config.color_calibration || {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            w: 1.0
          },
        });
      } else {
        console.warn('⚠️ App.tsx - 配置数据无效或缺少strips:', config);
        // 设置空的配置
        setLedStripStore({
          strips: [],
          colorCalibration: {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            w: 1.0
          },
        });
      }
    }).catch((error: any) => {
      console.error('Failed to read config:', error);
      // 设置空的配置
      setLedStripStore({
        strips: [],
        colorCalibration: {
          r: 1.0,
          g: 1.0,
          b: 1.0,
          w: 1.0
        },
      });
    });
  });

  return (
    <div class="h-screen bg-base-100 flex flex-col">
      {/* Fixed Navigation */}
      <div class="navbar bg-base-200 shadow-lg flex-shrink-0 z-50">
        <div class="navbar-start">
          <div class="dropdown dropdown-hover">
            <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16"></path>
              </svg>
            </div>
            <ul class="menu menu-sm dropdown-content z-[100] p-2 shadow bg-base-100 rounded-box w-52 border border-base-300">
              <li><A href="/info" class="text-base-content hover:bg-base-200">{t('nav.info')}</A></li>
              <li><A href="/led-strips-configuration" class="text-base-content hover:bg-base-200">{t('nav.ledConfiguration')}</A></li>
              <li><A href="/color-calibration" class="text-base-content hover:bg-base-200">{t('nav.colorCalibration')}</A></li>
              <li><A href="/led-strip-test" class="text-base-content hover:bg-base-200">{t('nav.ledTest')}</A></li>
              <li><A href="/settings" class="text-base-content hover:bg-base-200">{t('nav.settings')}</A></li>
            </ul>
          </div>
          <a class="btn btn-ghost text-xl text-primary font-bold">{t('nav.title')}</a>
        </div>
        <div class="navbar-center hidden lg:flex">
          <ul class="menu menu-horizontal px-1">
            <li><A href="/info" class="btn btn-ghost text-base-content hover:text-primary">{t('nav.info')}</A></li>
            <li><A href="/led-strips-configuration" class="btn btn-ghost text-base-content hover:text-primary">{t('nav.ledConfiguration')}</A></li>
            <li><A href="/color-calibration" class="btn btn-ghost text-base-content hover:text-primary">{t('nav.colorCalibration')}</A></li>
            <li><A href="/led-strip-test" class="btn btn-ghost text-base-content hover:text-primary">{t('nav.ledTest')}</A></li>
            <li><A href="/settings" class="btn btn-ghost text-base-content hover:text-primary">{t('nav.settings')}</A></li>
          </ul>
        </div>
        <div class="navbar-end">
          <div class="flex items-center gap-2">
            <div class="badge badge-primary badge-outline">
              v{appVersion().version}
            </div>
            {appVersion().is_dev && (
              <div class="badge badge-warning badge-outline">
                DEV
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Main Content - fills remaining height */}
      <main class="flex-1 container mx-auto px-2 sm:px-4 py-4 max-w-full overflow-x-auto min-h-0">
        {/* Routes are now handled by the Router component in index.tsx */}
        {props.children}
      </main>

      {/* Status Bar - fixed at bottom */}
      <StatusBar />
    </div>
  );
}

export default App;
