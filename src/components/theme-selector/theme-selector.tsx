import { createSignal, For, onMount, onCleanup } from 'solid-js';
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

// 获取主题颜色的工具函数 - 使用临时隐藏元素
const getThemeColors = (theme: DaisyUITheme): { primary: string; secondary: string; accent: string; base100: string; base200: string } => {
  // 创建一个临时的隐藏容器
  const tempContainer = document.createElement('div');
  tempContainer.setAttribute('data-theme', theme);
  tempContainer.style.position = 'absolute';
  tempContainer.style.left = '-9999px';
  tempContainer.style.visibility = 'hidden';
  tempContainer.style.pointerEvents = 'none';

  // 创建颜色测试元素
  const primaryEl = document.createElement('div');
  primaryEl.className = 'bg-primary';
  const secondaryEl = document.createElement('div');
  secondaryEl.className = 'bg-secondary';
  const accentEl = document.createElement('div');
  accentEl.className = 'bg-accent';
  const base100El = document.createElement('div');
  base100El.className = 'bg-base-100';
  const base200El = document.createElement('div');
  base200El.className = 'bg-base-200';

  tempContainer.appendChild(primaryEl);
  tempContainer.appendChild(secondaryEl);
  tempContainer.appendChild(accentEl);
  tempContainer.appendChild(base100El);
  tempContainer.appendChild(base200El);

  // 添加到body获取计算样式
  document.body.appendChild(tempContainer);

  const primaryColor = getComputedStyle(primaryEl).backgroundColor;
  const secondaryColor = getComputedStyle(secondaryEl).backgroundColor;
  const accentColor = getComputedStyle(accentEl).backgroundColor;
  const base100Color = getComputedStyle(base100El).backgroundColor;
  const base200Color = getComputedStyle(base200El).backgroundColor;

  // 立即清理
  document.body.removeChild(tempContainer);

  return {
    primary: primaryColor,
    secondary: secondaryColor,
    accent: accentColor,
    base100: base100Color,
    base200: base200Color
  };
};

// 主题颜色预览组件
const ThemeColorPreview = (props: { theme: DaisyUITheme; size?: 'sm' | 'md' }) => {
  const [colors, setColors] = createSignal<{ primary: string; secondary: string; accent: string; base100: string; base200: string } | null>(null);
  const size = props.size || 'md';
  const dotSize = size === 'sm' ? 'w-1.5 h-1.5' : 'w-2 h-2';
  const containerHeight = size === 'sm' ? 'h-6' : 'h-8';
  const containerPadding = size === 'sm' ? 'p-1' : 'p-1.5';

  onMount(() => {
    // 延迟获取颜色，避免阻塞渲染
    setTimeout(() => {
      const themeColors = getThemeColors(props.theme);
      setColors(themeColors);
    }, 0);
  });

  return (
    <div
      class={`flex items-center gap-1 flex-shrink-0 ${containerPadding} ${containerHeight} rounded-md border border-base-300/50`}
      style={{ 'background-color': colors()?.base100 || 'hsl(var(--b1))' }}
    >
      {/* 背景色条 */}
      <div
        class="w-1 h-full rounded-sm flex-shrink-0"
        style={{ 'background-color': colors()?.base200 || 'hsl(var(--b2))' }}
      ></div>

      {/* 主题色圆点 */}
      <div class="flex gap-0.5">
        <div
          class={`${dotSize} rounded-full`}
          style={{ 'background-color': colors()?.primary || 'hsl(var(--p))' }}
        ></div>
        <div
          class={`${dotSize} rounded-full`}
          style={{ 'background-color': colors()?.secondary || 'hsl(var(--s))' }}
        ></div>
        <div
          class={`${dotSize} rounded-full`}
          style={{ 'background-color': colors()?.accent || 'hsl(var(--a))' }}
        ></div>
      </div>
    </div>
  );
};

export const ThemeSelector = () => {
  const { t } = useLanguage();
  const [isOpen, setIsOpen] = createSignal(false);
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
                  {(theme) => (
                    <button
                      class={`w-full flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 transition-colors ${themeStore.currentTheme() === theme ? 'ring-2 ring-primary ring-offset-2 ring-offset-base-100' : ''
                        }`}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleThemeChange(theme);
                      }}
                    >
                      <ThemeColorPreview theme={theme} />
                      <div class="flex-1 text-left">
                        <div class="font-medium text-base-content">{getThemeDisplayName(theme, t)}</div>
                      </div>
                      {themeStore.currentTheme() === theme && (
                        <svg class="w-4 h-4 text-primary flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                          <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                        </svg>
                      )}
                    </button>
                  )}
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
                  {(theme) => (
                    <button
                      class={`w-full flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 transition-colors ${themeStore.currentTheme() === theme ? 'ring-2 ring-primary ring-offset-2 ring-offset-base-100' : ''
                        }`}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleThemeChange(theme);
                      }}
                    >
                      <ThemeColorPreview theme={theme} />
                      <div class="flex-1 text-left">
                        <div class="font-medium text-base-content">{getThemeDisplayName(theme, t)}</div>
                      </div>
                      {themeStore.currentTheme() === theme && (
                        <svg class="w-4 h-4 text-primary flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                          <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                        </svg>
                      )}
                    </button>
                  )}
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
                      {(theme) => (
                        <button
                          class={`w-full flex items-center gap-2 p-2 rounded-lg hover:bg-base-200 transition-colors text-sm ${nightModeTheme() === theme ? 'ring-2 ring-primary ring-offset-1 ring-offset-base-100' : ''
                            }`}
                          onClick={(e) => {
                            e.stopPropagation();
                            handleNightModeThemeChange(theme);
                          }}
                        >
                          <ThemeColorPreview theme={theme} size="sm" />
                          <div class="flex-1 text-left">
                            <div class="font-medium text-base-content">{getThemeDisplayName(theme, t)}</div>
                          </div>
                          {nightModeTheme() === theme && (
                            <svg class="w-3 h-3 text-primary flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                            </svg>
                          )}
                        </button>
                      )}
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
                      {(theme) => (
                        <button
                          class={`w-full flex items-center gap-2 p-2 rounded-lg hover:bg-base-200 transition-colors text-sm ${nightModeTheme() === theme ? 'ring-2 ring-primary ring-offset-1 ring-offset-base-100' : ''
                            }`}
                          onClick={(e) => {
                            e.stopPropagation();
                            handleNightModeThemeChange(theme);
                          }}
                        >
                          <ThemeColorPreview theme={theme} size="sm" />
                          <div class="flex-1 text-left">
                            <div class="font-medium text-base-content">{getThemeDisplayName(theme, t)}</div>
                          </div>
                          {nightModeTheme() === theme && (
                            <svg class="w-3 h-3 text-primary flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                            </svg>
                          )}
                        </button>
                      )}
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
