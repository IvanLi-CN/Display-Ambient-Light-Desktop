import { createSignal, createEffect, onMount, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useLanguage } from '../../i18n/index';

interface AmbientLightState {
  enabled: boolean;
}

export const AmbientLightControl = () => {
  const { t } = useLanguage();
  const [ambientLightEnabled, setAmbientLightEnabled] = createSignal(true);
  const [loading, setLoading] = createSignal(false);
  const [message, setMessage] = createSignal<{ type: 'success' | 'error'; text: string } | null>(null);

  // Load ambient light state on mount
  onMount(async () => {
    try {
      const state = await invoke<AmbientLightState>('get_ambient_light_state');
      setAmbientLightEnabled(state.enabled);
    } catch (error) {
      console.error('Failed to load ambient light state:', error);
    }
  });

  // Listen for ambient light state changes from tray menu
  createEffect(() => {
    const unlisten = listen<AmbientLightState>('ambient_light_state_changed', (event) => {
      console.log('Ambient light state changed from tray:', event.payload);
      setAmbientLightEnabled(event.payload.enabled);

      // Show notification message
      showMessage('success', event.payload.enabled ? t('ambientLight.statusEnabled') : t('ambientLight.statusDisabled'));
    });

    onCleanup(() => {
      unlisten.then((unlisten) => unlisten());
    });
  });

  // Handle ambient light toggle
  const handleAmbientLightToggle = async () => {
    setLoading(true);
    try {
      const newState = !ambientLightEnabled();
      await invoke('set_ambient_light_enabled', { enabled: newState });
      setAmbientLightEnabled(newState);

      // Update tray menu to reflect new state
      await invoke('update_tray_menu');

      showMessage('success', newState ? t('ambientLight.statusEnabled') : t('ambientLight.statusDisabled'));
    } catch (error) {
      console.error('Failed to toggle ambient light:', error);
      showMessage('error', t('ambientLight.toggleFailed'));
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
    <div class="card bg-base-100 shadow-lg border border-base-300">
      <div class="card-body p-6">
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-3">
            <div class="flex-shrink-0">
              <svg class="w-6 h-6 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"></path>
              </svg>
            </div>
            <div>
              <h2 class="text-xl font-semibold text-base-content">{t('ambientLight.title')}</h2>
              <p class="text-sm text-base-content/70 mt-1">{t('ambientLight.description')}</p>
            </div>
          </div>

          {/* Status Badge */}
          <div class={`badge ${ambientLightEnabled() ? 'badge-success' : 'badge-error'} gap-2`}>
            <div class="w-2 h-2 rounded-full bg-current"></div>
            {ambientLightEnabled() ? t('ambientLight.enabled') : t('ambientLight.disabled')}
          </div>
        </div>

        {/* Status Message */}
        {message() && (
          <div class={`alert ${message()!.type === 'success' ? 'alert-success' : 'alert-error'} mb-4`}>
            <svg class="w-6 h-6 stroke-current flex-shrink-0" fill="none" viewBox="0 0 24 24">
              {message()!.type === 'success' ? (
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              ) : (
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              )}
            </svg>
            <span>{message()!.text}</span>
          </div>
        )}

        {/* Main Control */}
        <div class="bg-base-200 rounded-lg p-4">
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3 mb-2">
                <span class="text-base font-medium text-base-content">
                  {t('ambientLight.title')}
                </span>
                {loading() && (
                  <span class="loading loading-spinner loading-sm"></span>
                )}
              </div>
              <p class="text-sm text-base-content/70">
                {ambientLightEnabled()
                  ? t('ambientLight.descriptionEnabled')
                  : t('ambientLight.descriptionDisabled')
                }
              </p>
            </div>

            <div class="flex-shrink-0 ml-4">
              <input
                type="checkbox"
                class="toggle toggle-primary toggle-lg"
                checked={ambientLightEnabled()}
                disabled={loading()}
                onChange={handleAmbientLightToggle}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
