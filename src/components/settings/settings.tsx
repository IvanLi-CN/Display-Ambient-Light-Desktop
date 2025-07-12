import { createSignal, createEffect, onMount } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { useLanguage } from '../../i18n/index';
import { AmbientLightControl } from '../ambient-light-control/ambient-light-control';
import { ThemeSelector } from '../theme-selector/theme-selector';


interface AutoStartConfig {
  enabled: boolean;
}

export const Settings = () => {
  const { t, locale, setLocale } = useLanguage();
  const [autoStartEnabled, setAutoStartEnabled] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [message, setMessage] = createSignal<{ type: 'success' | 'error'; text: string } | null>(null);



  // Load auto start status and user preferences on mount
  onMount(async () => {
    try {
      const config = await invoke<AutoStartConfig>('get_auto_start_config');
      setAutoStartEnabled(config.enabled);
    } catch (error) {
      console.error('Failed to load auto start config:', error);
    }


  });

  // Handle language change
  const handleLanguageChange = async (newLocale: 'zh-CN' | 'en-US') => {
    try {
      // Update frontend language
      setLocale(newLocale);

      // Update backend language setting
      await invoke('set_current_language', { language: newLocale });

      // Update tray menu with new language
      await invoke('update_tray_menu');

      showMessage('success', t('settings.languageDescription'));
    } catch (error) {
      console.error('Failed to change language:', error);
      showMessage('error', 'Failed to change language');
    }
  };

  // Handle auto start toggle
  const handleAutoStartToggle = async () => {
    setLoading(true);
    try {
      const newState = !autoStartEnabled();
      await invoke('set_auto_start_enabled', { enabled: newState });
      setAutoStartEnabled(newState);

      // Update tray menu to reflect new state
      await invoke('update_tray_menu');

      showMessage('success', newState ? t('settings.autoStartEnabled') : t('settings.autoStartDisabled'));
    } catch (error) {
      console.error('Failed to toggle auto start:', error);
      showMessage('error', t('settings.autoStartError'));
    } finally {
      setLoading(false);
    }
  };

  // Show message helper
  const showMessage = (type: 'success' | 'error', text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 3000);
  };





  return (
    <div class="container mx-auto p-6 max-w-4xl">
      <div class="mb-6">
        <h1 class="text-3xl font-bold text-base-content mb-2">{t('settings.title')}</h1>
      </div>

      {/* Message Alert */}
      {message() && (
        <div class={`alert ${message()!.type === 'success' ? 'alert-success' : 'alert-error'} mb-6`}>
          <svg class="w-6 h-6 shrink-0 stroke-current" fill="none" viewBox="0 0 24 24">
            {message()!.type === 'success' ? (
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            ) : (
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            )}
          </svg>
          <span>{message()!.text}</span>
        </div>
      )}

      <div class="grid gap-6">
        {/* Ambient Light Control */}
        <AmbientLightControl />

        {/* Appearance Settings */}
        <div class="settings-card">
          <div class="card-body">
            <h2 class="card-title text-xl mb-4 flex items-center">
              <svg class="w-6 h-6 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zM21 5a2 2 0 00-2-2h-4a2 2 0 00-2 2v12a4 4 0 004 4h4a2 2 0 002-2V5z"></path>
              </svg>
              {t('settings.appearance')}
            </h2>

            {/* Theme Selection */}
            <div class="mb-6">
              <ThemeSelector />
            </div>

            {/* Language Selection */}
            <div class="form-control">
              <label class="label">
                <span class="label-text text-base font-medium">{t('settings.language')}</span>
              </label>
              <div class="flex flex-col gap-2">
                <p class="text-sm text-base-content/70 mb-3">{t('settings.languageDescription')}</p>
                <div class="flex gap-3">
                  <button
                    class={`btn ${locale() === 'zh-CN' ? 'btn-primary' : 'btn-outline'} flex-1`}
                    onClick={() => handleLanguageChange('zh-CN')}
                  >
                    中文
                  </button>
                  <button
                    class={`btn ${locale() === 'en-US' ? 'btn-primary' : 'btn-outline'} flex-1`}
                    onClick={() => handleLanguageChange('en-US')}
                  >
                    English
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* System Settings */}
        <div class="settings-card">
          <div class="card-body">
            <h2 class="card-title text-xl mb-4 flex items-center">
              <svg class="w-6 h-6 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"></path>
              </svg>
              {t('settings.system')}
            </h2>

            {/* Auto Start Setting */}
            <div class="form-control">
              <div class="toggle-right-container cursor-pointer">
                <div class="flex flex-col items-start flex-1">
                  <span class="label-text text-base font-medium">{t('settings.autoStart')}</span>
                  <span class="text-sm text-base-content/70 mt-1">{t('settings.autoStartDescription')}</span>
                </div>
                <input
                  type="checkbox"
                  class="toggle toggle-primary"
                  checked={autoStartEnabled()}
                  onChange={handleAutoStartToggle}
                  disabled={loading()}
                />
              </div>
            </div>
          </div>
        </div>



        {/* About Section */}
        <div class="settings-card">
          <div class="card-body">
            <div class="flex items-center justify-between">
              <div class="flex items-center">
                <svg class="w-6 h-6 mr-3 text-base-content/70" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                </svg>
                <div>
                  <h3 class="text-lg font-medium text-base-content">{t('settings.about')}</h3>
                  <p class="text-sm text-base-content/70">Ambient Light Control v2.0.0-alpha</p>
                </div>
              </div>
              <button
                class="btn btn-outline btn-sm"
                onClick={async () => {
                  try {
                    await invoke('show_about_window');
                  } catch (error) {
                    console.error('Failed to show about window:', error);
                  }
                }}
              >
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                </svg>
                {t('settings.about')}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
