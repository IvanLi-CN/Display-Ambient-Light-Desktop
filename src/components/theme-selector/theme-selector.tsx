import { createSignal, For, onMount, onCleanup } from 'solid-js';
import { themeStore, DaisyUITheme, AVAILABLE_THEMES } from '../../stores/theme.store';
import { useLanguage } from '../../i18n/index';

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

export const ThemeSelector = () => {
  const { t } = useLanguage();
  const [isOpen, setIsOpen] = createSignal(false);
  let containerRef: HTMLDivElement | undefined;

  const handleThemeChange = (theme: DaisyUITheme) => {
    themeStore.setCurrentTheme(theme);
    setIsOpen(false);
  };

  const currentThemeName = () => getThemeDisplayName(themeStore.currentTheme(), t);

  // 点击外部关闭下拉菜单
  const handleClickOutside = (event: MouseEvent) => {
    if (containerRef && !containerRef.contains(event.target as Node)) {
      setIsOpen(false);
    }
  };

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
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
            setIsOpen(!isOpen());
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
          <div class="absolute top-full left-0 right-0 z-[1000] mt-1 bg-base-100 rounded-box shadow-lg border border-base-300 max-h-60 overflow-y-auto">
            <div class="p-2">
              <For each={AVAILABLE_THEMES}>
                {(theme) => (
                  <button
                    class={`w-full flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 transition-colors ${themeStore.currentTheme() === theme ? 'bg-primary/10 text-primary' : ''
                      }`}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleThemeChange(theme);
                    }}
                  >
                    <div class="flex gap-1 flex-shrink-0">
                      <div class="w-2 h-2 rounded-full bg-primary opacity-80"></div>
                      <div class="w-2 h-2 rounded-full bg-secondary opacity-80"></div>
                      <div class="w-2 h-2 rounded-full bg-accent opacity-80"></div>
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
                )}
              </For>
            </div>
          </div>
        )}
      </div>

      <label class="label">
        <span class="label-text-alt text-base-content/60">{t('settings.themeDescription')}</span>
      </label>
    </div>
  );
};
