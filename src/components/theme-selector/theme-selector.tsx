import { createSignal, For, onMount, onCleanup, createEffect } from 'solid-js';
import { themeStore, DaisyUITheme, AVAILABLE_THEMES } from '../../stores/theme.store';
import { useLanguage } from '../../i18n/index';
import { userPreferencesStore } from '../../stores/user-preferences.store';

// 主题信息映射
const getThemeDisplayName = (theme: DaisyUITheme, t: any) => {
  const nameMap: Record<DaisyUITheme, string> = {
    light: t('settings.themeLight'),
    dark: t('settings.themeDark'),
    cupcake: 'Cupcake',
    bumblebee: 'Bumblebee',
    emerald: 'Emerald',
    corporate: 'Corporate',
    synthwave: 'Synthwave',
    retro: 'Retro',
    cyberpunk: 'Cyberpunk',
    valentine: 'Valentine',
    halloween: 'Halloween',
    garden: 'Garden',
    forest: 'Forest',
    aqua: 'Aqua',
    lofi: 'Lo-Fi',
    pastel: 'Pastel',
    fantasy: 'Fantasy',
    wireframe: 'Wireframe',
    black: 'Black',
    luxury: 'Luxury',
    dracula: 'Dracula',
    cmyk: 'CMYK',
    autumn: 'Autumn',
    business: 'Business',
    acid: 'Acid',
    lemonade: 'Lemonade',
    night: 'Night',
    coffee: 'Coffee',
    winter: 'Winter',
    dim: 'Dim',
    nord: 'Nord',
    sunset: 'Sunset',
    caramellatte: 'Caramel Latte',
    abyss: 'Abyss',
    silk: 'Silk',
  };

  return nameMap[theme] || theme;
};

// 主题分类
const LIGHT_THEMES: DaisyUITheme[] = [
  'light', 'cupcake', 'bumblebee', 'emerald', 'corporate', 'retro', 'valentine',
  'garden', 'aqua', 'lofi', 'pastel', 'fantasy', 'wireframe', 'cmyk', 'autumn',
  'acid', 'lemonade', 'winter', 'nord', 'sunset', 'caramellatte', 'silk'
];

const DARK_THEMES: DaisyUITheme[] = [
  'dark', 'synthwave', 'cyberpunk', 'halloween', 'forest', 'black', 'luxury',
  'dracula', 'business', 'night', 'coffee', 'dim', 'abyss'
];

// 获取主题颜色的工具函数
const getThemeColors = (theme: DaisyUITheme): { primary: string; secondary: string; accent: string } => {
  // 创建一个临时元素来获取主题颜色
  const tempDiv = document.createElement('div');
  tempDiv.setAttribute('data-theme', theme);
  tempDiv.style.position = 'absolute';
  tempDiv.style.visibility = 'hidden';
  tempDiv.style.pointerEvents = 'none';

  // 添加到DOM中以获取计算样式
  document.body.appendChild(tempDiv);

  // 创建子元素来获取颜色
  const primaryEl = document.createElement('div');
  primaryEl.className = 'bg-primary';
  const secondaryEl = document.createElement('div');
  secondaryEl.className = 'bg-secondary';
  const accentEl = document.createElement('div');
  accentEl.className = 'bg-accent';

  tempDiv.appendChild(primaryEl);
  tempDiv.appendChild(secondaryEl);
  tempDiv.appendChild(accentEl);

  // 获取计算样式
  const primaryColor = getComputedStyle(primaryEl).backgroundColor;
  const secondaryColor = getComputedStyle(secondaryEl).backgroundColor;
  const accentColor = getComputedStyle(accentEl).backgroundColor;

  // 清理DOM
  document.body.removeChild(tempDiv);

  return {
    primary: primaryColor,
    secondary: secondaryColor,
    accent: accentColor
  };
};

export const ThemeSelector = () => {
  const { t } = useLanguage();
  const [isOpen, setIsOpen] = createSignal(false);
  const [themeColors, setThemeColors] = createSignal<Record<DaisyUITheme, { primary: string; secondary: string; accent: string }>>({} as any);
  const [nightModeEnabled, setNightModeEnabled] = createSignal(false);
  const [nightModeTheme, setNightModeTheme] = createSignal<DaisyUITheme>('dark');
  const [isNightModeOpen, setIsNightModeOpen] = createSignal(false);
  let containerRef: HTMLDivElement | undefined;
  let nightModeContainerRef: HTMLDivElement | undefined;
  let dropdownRef: HTMLDivElement | undefined;
  let nightModeDropdownRef: HTMLDivElement | undefined;

  const handleThemeChange = (theme: DaisyUITheme) => {
    themeStore.setCurrentTheme(theme);
    setIsOpen(false);
  };

  const handleNightModeThemeChange = async (theme: DaisyUITheme) => {
    try {
      await userPreferencesStore.updateNightModeTheme(theme);
      setNightModeTheme(theme);
      setIsNightModeOpen(false);
      // Refresh effective theme
      await themeStore.refreshEffectiveTheme();
    } catch (error) {
      console.error('Failed to update night mode theme:', error);
    }
  };

  const handleNightModeToggle = async () => {
    try {
      const newEnabled = !nightModeEnabled();
      await userPreferencesStore.updateNightModeThemeEnabled(newEnabled);
      setNightModeEnabled(newEnabled);
      // Refresh effective theme
      await themeStore.refreshEffectiveTheme();
    } catch (error) {
      console.error('Failed to toggle night mode theme:', error);
    }
  };

  const currentThemeName = () => getThemeDisplayName(themeStore.currentTheme(), t);
  const nightModeThemeName = () => getThemeDisplayName(nightModeTheme(), t);

  // 滚动到当前选中的主题
  const scrollToCurrentTheme = (dropdown: HTMLDivElement, currentTheme: DaisyUITheme) => {
    setTimeout(() => {
      const selectedButton = dropdown.querySelector(`[data-theme="${currentTheme}"]`) as HTMLButtonElement;
      if (selectedButton) {
        selectedButton.scrollIntoView({
          behavior: 'smooth',
          block: 'center'
        });
      }
    }, 50); // 小延迟确保DOM已渲染
  };

  // 点击外部关闭下拉菜单
  const handleClickOutside = (event: MouseEvent) => {
    if (containerRef && !containerRef.contains(event.target as Node)) {
      setIsOpen(false);
    }
    if (nightModeContainerRef && !nightModeContainerRef.contains(event.target as Node)) {
      setIsNightModeOpen(false);
    }
  };

  // 预加载所有主题颜色
  const preloadThemeColors = () => {
    const colors: Record<DaisyUITheme, { primary: string; secondary: string; accent: string }> = {} as any;
    AVAILABLE_THEMES.forEach(theme => {
      colors[theme] = getThemeColors(theme);
    });
    setThemeColors(colors);
  };

  // 初始化夜间模式设置
  const initializeNightModeSettings = async () => {
    try {
      const enabled = await userPreferencesStore.getNightModeThemeEnabled();
      const theme = await userPreferencesStore.getNightModeTheme();
      setNightModeEnabled(enabled);
      if (theme && AVAILABLE_THEMES.includes(theme as DaisyUITheme)) {
        setNightModeTheme(theme as DaisyUITheme);
      }
    } catch (error) {
      console.error('Failed to initialize night mode settings:', error);
    }
  };

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    // 延迟预加载颜色，避免阻塞初始渲染
    setTimeout(preloadThemeColors, 100);
    // 初始化夜间模式设置
    initializeNightModeSettings();
  });

  onCleanup(() => {
    document.removeEventListener('click', handleClickOutside);
  });

  return (
    <div class="form-control w-full">
      <label class="label">
        <span class="label-text text-base font-medium">{t('settings.theme')}</span>
      </label>

      <div class="relative w-full" ref={containerRef}>
        <button
          class="btn btn-outline w-full justify-between"
          onClick={(e) => {
            e.stopPropagation();
            const newIsOpen = !isOpen();
            setIsOpen(newIsOpen);
            if (newIsOpen && dropdownRef) {
              scrollToCurrentTheme(dropdownRef, themeStore.currentTheme());
            }
          }}
        >
          <div class="flex items-center gap-3">
            <div class="flex gap-1">
              <div class="w-3 h-3 rounded-full bg-primary"></div>
              <div class="w-3 h-3 rounded-full bg-secondary"></div>
              <div class="w-3 h-3 rounded-full bg-accent"></div>
            </div>
            <span>{currentThemeName()}</span>
          </div>
          <svg
            class={`w-4 h-4 transition-transform ${isOpen() ? 'rotate-180' : ''}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
          </svg>
        </button>

        {isOpen() && (
          <div ref={dropdownRef} class="absolute top-full left-0 right-0 z-[1000] mt-1 bg-base-100 rounded-box shadow-lg border border-base-300 max-h-60 overflow-y-auto">
            <div class="p-2">
              {/* 亮色主题组 */}
              <div class="mb-3">
                <div class="text-xs font-medium text-base-content/60 px-2 py-1 mb-1">
                  {t('settings.lightThemes')}
                </div>
                <For each={LIGHT_THEMES}>
                  {(theme) => {
                    const colors = () => themeColors()[theme];
                    return (
                      <button
                        class={`w-full flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 transition-colors ${themeStore.currentTheme() === theme ? 'bg-primary/10 text-primary' : ''
                          }`}
                        data-theme={theme}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleThemeChange(theme);
                        }}
                      >
                        <div class="flex gap-1 flex-shrink-0">
                          <div
                            class="w-2 h-2 rounded-full opacity-80"
                            style={{ 'background-color': colors()?.primary || 'hsl(var(--p))' }}
                          ></div>
                          <div
                            class="w-2 h-2 rounded-full opacity-80"
                            style={{ 'background-color': colors()?.secondary || 'hsl(var(--s))' }}
                          ></div>
                          <div
                            class="w-2 h-2 rounded-full opacity-80"
                            style={{ 'background-color': colors()?.accent || 'hsl(var(--a))' }}
                          ></div>
                        </div>
                        <div class="flex-1 text-left">
                          <div class="font-medium">{getThemeDisplayName(theme, t)}</div>
                        </div>
                        {themeStore.currentTheme() === theme && (
                          <svg class="w-4 h-4 text-primary" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                          </svg>
                        )}
                      </button>
                    );
                  }}
                </For>
              </div>

              {/* 分隔线 */}
              <div class="divider my-2"></div>

              {/* 暗色主题组 */}
              <div>
                <div class="text-xs font-medium text-base-content/60 px-2 py-1 mb-1">
                  {t('settings.darkThemes')}
                </div>
                <For each={DARK_THEMES}>
                  {(theme) => {
                    const colors = () => themeColors()[theme];
                    return (
                      <button
                        class={`w-full flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 transition-colors ${themeStore.currentTheme() === theme ? 'bg-primary/10 text-primary' : ''
                          }`}
                        data-theme={theme}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleThemeChange(theme);
                        }}
                      >
                        <div class="flex gap-1 flex-shrink-0">
                          <div
                            class="w-2 h-2 rounded-full opacity-80"
                            style={{ 'background-color': colors()?.primary || 'hsl(var(--p))' }}
                          ></div>
                          <div
                            class="w-2 h-2 rounded-full opacity-80"
                            style={{ 'background-color': colors()?.secondary || 'hsl(var(--s))' }}
                          ></div>
                          <div
                            class="w-2 h-2 rounded-full opacity-80"
                            style={{ 'background-color': colors()?.accent || 'hsl(var(--a))' }}
                          ></div>
                        </div>
                        <div class="flex-1 text-left">
                          <div class="font-medium">{getThemeDisplayName(theme, t)}</div>
                        </div>
                        {themeStore.currentTheme() === theme && (
                          <svg class="w-4 h-4 text-primary" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                          </svg>
                        )}
                      </button>
                    );
                  }}
                </For>
              </div>
            </div>
          </div>
        )}
      </div>

      <label class="label">
        <span class="label-text-alt text-base-content/60">{t('settings.themeDescription')}</span>
      </label>

      {/* 夜间模式主题配置 */}
      <div class="mt-4 p-4 bg-base-200 rounded-lg">
        <div class="flex items-center justify-between mb-3">
          <div>
            <h4 class="font-medium text-base-content">{t('settings.nightModeTheme')}</h4>
            <p class="text-sm text-base-content/70">{t('settings.nightModeThemeDescription')}</p>
          </div>
          <input
            type="checkbox"
            class="toggle toggle-primary"
            checked={nightModeEnabled()}
            onChange={handleNightModeToggle}
          />
        </div>

        {nightModeEnabled() && (
          <div class="relative w-full" ref={nightModeContainerRef}>
            <button
              class="btn btn-outline btn-sm w-full justify-between"
              onClick={(e) => {
                e.stopPropagation();
                const newIsOpen = !isNightModeOpen();
                setIsNightModeOpen(newIsOpen);
                if (newIsOpen && nightModeDropdownRef) {
                  scrollToCurrentTheme(nightModeDropdownRef, nightModeTheme());
                }
              }}
            >
              <div class="flex items-center gap-2">
                <div class="flex gap-1">
                  <div class="w-2 h-2 rounded-full bg-primary"></div>
                  <div class="w-2 h-2 rounded-full bg-secondary"></div>
                  <div class="w-2 h-2 rounded-full bg-accent"></div>
                </div>
                <span class="text-sm">{nightModeThemeName()}</span>
              </div>
              <svg
                class={`w-3 h-3 transition-transform ${isNightModeOpen() ? 'rotate-180' : ''}`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
              </svg>
            </button>

            {isNightModeOpen() && (
              <div ref={nightModeDropdownRef} class="absolute top-full left-0 right-0 z-[1001] mt-1 bg-base-100 rounded-box shadow-lg border border-base-300 max-h-48 overflow-y-auto">
                <div class="p-2">
                  {/* 亮色主题组 */}
                  <div class="mb-2">
                    <div class="text-xs font-medium text-base-content/60 px-2 py-1 mb-1">
                      {t('settings.lightThemes')}
                    </div>
                    <For each={LIGHT_THEMES}>
                      {(theme) => {
                        const colors = () => themeColors()[theme];
                        return (
                          <button
                            class={`w-full flex items-center gap-2 p-2 rounded-lg hover:bg-base-200 transition-colors text-sm ${nightModeTheme() === theme ? 'bg-primary/10 text-primary' : ''
                              }`}
                            data-theme={theme}
                            onClick={(e) => {
                              e.stopPropagation();
                              handleNightModeThemeChange(theme);
                            }}
                          >
                            <div class="flex gap-1 flex-shrink-0">
                              <div
                                class="w-1.5 h-1.5 rounded-full opacity-80"
                                style={{ 'background-color': colors()?.primary || 'hsl(var(--p))' }}
                              ></div>
                              <div
                                class="w-1.5 h-1.5 rounded-full opacity-80"
                                style={{ 'background-color': colors()?.secondary || 'hsl(var(--s))' }}
                              ></div>
                              <div
                                class="w-1.5 h-1.5 rounded-full opacity-80"
                                style={{ 'background-color': colors()?.accent || 'hsl(var(--a))' }}
                              ></div>
                            </div>
                            <div class="flex-1 text-left">
                              <div class="font-medium">{getThemeDisplayName(theme, t)}</div>
                            </div>
                            {nightModeTheme() === theme && (
                              <svg class="w-3 h-3 text-primary" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                              </svg>
                            )}
                          </button>
                        );
                      }}
                    </For>
                  </div>

                  {/* 分隔线 */}
                  <div class="divider my-1"></div>

                  {/* 暗色主题组 */}
                  <div>
                    <div class="text-xs font-medium text-base-content/60 px-2 py-1 mb-1">
                      {t('settings.darkThemes')}
                    </div>
                    <For each={DARK_THEMES}>
                      {(theme) => {
                        const colors = () => themeColors()[theme];
                        return (
                          <button
                            class={`w-full flex items-center gap-2 p-2 rounded-lg hover:bg-base-200 transition-colors text-sm ${nightModeTheme() === theme ? 'bg-primary/10 text-primary' : ''
                              }`}
                            data-theme={theme}
                            onClick={(e) => {
                              e.stopPropagation();
                              handleNightModeThemeChange(theme);
                            }}
                          >
                            <div class="flex gap-1 flex-shrink-0">
                              <div
                                class="w-1.5 h-1.5 rounded-full opacity-80"
                                style={{ 'background-color': colors()?.primary || 'hsl(var(--p))' }}
                              ></div>
                              <div
                                class="w-1.5 h-1.5 rounded-full opacity-80"
                                style={{ 'background-color': colors()?.secondary || 'hsl(var(--s))' }}
                              ></div>
                              <div
                                class="w-1.5 h-1.5 rounded-full opacity-80"
                                style={{ 'background-color': colors()?.accent || 'hsl(var(--a))' }}
                              ></div>
                            </div>
                            <div class="flex-1 text-left">
                              <div class="font-medium">{getThemeDisplayName(theme, t)}</div>
                            </div>
                            {nightModeTheme() === theme && (
                              <svg class="w-3 h-3 text-primary" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                              </svg>
                            )}
                          </button>
                        );
                      }}
                    </For>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
