import { render } from 'solid-js/web';
import { createSignal, createEffect, onMount } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { LanguageProvider, useLanguage } from './i18n/index';
import { themeStore } from './stores/theme.store';

const AboutWindow = () => {
  const { t } = useLanguage();
  const [appVersion, setAppVersion] = createSignal('2.0.0-alpha');

  // Get app version on mount
  onMount(async () => {
    try {
      const version = await invoke<string>('get_app_version_string');
      setAppVersion(version);
    } catch (error) {
      console.warn('Failed to get app version from Tauri, using default');
    }
  });

  // Handle external link clicks
  const openExternalLink = async (url: string) => {
    try {
      await invoke('open_external_url', { url });
    } catch (error) {
      console.error('Failed to open external URL:', error);
      // Fallback to window.open
      window.open(url, '_blank');
    }
  };

  return (
    <div class="min-h-screen bg-base-200 flex items-center justify-center p-4" data-theme={themeStore.currentTheme()}>
      <div class="w-full max-w-sm">
        <div class="bg-base-100 rounded-lg shadow-xl p-5">
          {/* Header */}
          <div class="text-center mb-4">
            <div class="w-14 h-14 mx-auto mb-2 bg-primary rounded-lg flex items-center justify-center">
              <svg class="w-8 h-8 text-primary-content" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"></path>
              </svg>
            </div>
            <h1 class="text-lg font-bold text-base-content mb-1">{t('about.title')}</h1>
            <p class="text-xs text-base-content/70 leading-relaxed">{t('about.description')}</p>
          </div>

          {/* App Information */}
          <div class="space-y-2 mb-4">
            <div class="flex justify-between items-center">
              <span class="text-sm text-base-content/70">{t('about.version')}:</span>
              <span class="font-mono text-xs bg-base-200 px-2 py-1 rounded">{appVersion()}</span>
            </div>

            <div class="flex justify-between items-center">
              <span class="text-sm text-base-content/70">{t('about.author')}:</span>
              <span class="text-sm text-base-content">Ivan Li</span>
            </div>

            <div class="flex justify-between items-center">
              <span class="text-sm text-base-content/70">{t('about.license')}:</span>
              <span class="text-sm text-base-content">MIT</span>
            </div>
          </div>

          {/* Action Buttons */}
          <div class="flex flex-col gap-2 mb-3">
            <button
              class="btn btn-outline btn-xs h-8"
              onClick={() => openExternalLink('https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop')}
            >
              <svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
              {t('about.openRepository')}
            </button>

            <button
              class="btn btn-outline btn-xs h-8"
              onClick={() => openExternalLink('https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop')}
            >
              <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
              </svg>
              {t('about.openHomepage')}
            </button>
          </div>

          {/* Footer */}
          <div class="pt-2 border-t border-base-300 text-center">
            <p class="text-xs text-base-content/50">
              © 2024 Ivan Li. All rights reserved.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

const App = () => {
  return (
    <LanguageProvider>
      <AboutWindow />
    </LanguageProvider>
  );
};

render(() => <App />, document.getElementById('about-root')!);
