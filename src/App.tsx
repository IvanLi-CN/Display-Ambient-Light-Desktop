import { Routes, Route, useLocation, A, Navigate } from '@solidjs/router';
import { LedStripConfiguration } from './components/led-strip-configuration/led-strip-configuration';
import { WhiteBalance } from './components/white-balance/white-balance';
import { LedStripTest } from './components/led-strip-test/led-strip-test';
import { Settings } from './components/settings/settings';
import { createEffect, createSignal } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { setLedStripStore } from './stores/led-strip.store';
import { LedStripConfigContainer } from './models/led-strip-config';
import { InfoIndex } from './components/info/info-index';
import { useLanguage } from './i18n/index';
import { AppVersion } from './models/app-version.model';

function App() {
  const location = useLocation();
  const [previousPath, setPreviousPath] = createSignal<string>('');
  const { t } = useLanguage();
  const [appVersion, setAppVersion] = createSignal<AppVersion>({ version: '1.0.0', is_dev: false });

  // Get app version on mount
  createEffect(() => {
    invoke<AppVersion>('get_app_version').then((version) => {
      setAppVersion(version);
    }).catch((error) => {
      console.error('Failed to get app version:', error);
    });
  });



  // Monitor route changes and cleanup LED tests when leaving the test page
  createEffect(() => {
    const currentPath = location.pathname;
    const prevPath = previousPath();

    // Check if we're leaving the LED test page
    const isLeavingTestPage = prevPath === '/led-strip-test' && currentPath !== '/led-strip-test';

    if (isLeavingTestPage) {
      // The LED test component will handle stopping the test effect via onCleanup
      // We just need to ensure test mode is disabled to resume normal LED publishing
      invoke('disable_test_mode').catch((error) => {
        console.error('Failed to disable test mode:', error);
      });
    }

    // Update previousPath after the condition check
    setPreviousPath(currentPath);
  });

  createEffect(() => {
    invoke<LedStripConfigContainer>('read_config').then((config) => {
      setLedStripStore({
        strips: config.strips,
        mappers: config.mappers,
        colorCalibration: config.color_calibration,
      });
    }).catch((error) => {
      console.error('Failed to read config:', error);
    });
  });

  return (
    <div class="h-screen bg-base-100 flex flex-col" data-theme="dark">
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
              <li><A href="/white-balance" class="text-base-content hover:bg-base-200">{t('nav.whiteBalance')}</A></li>
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
            <li><A href="/white-balance" class="btn btn-ghost text-base-content hover:text-primary">{t('nav.whiteBalance')}</A></li>
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
        <Routes>
          <Route path="/" element={<Navigate href="/info" />} />
          <Route path="/info" component={InfoIndex} />
          <Route path="/led-strips-configuration" component={LedStripConfiguration} />
          <Route path="/white-balance" component={WhiteBalance} />
          <Route path="/led-strip-test" element={<LedStripTest />} />
          <Route path="/settings" component={Settings} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
