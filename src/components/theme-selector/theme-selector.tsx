import { createSignal, For, onMount, onCleanup } from 'solid-js';
import { themeStore, DaisyUITheme, AVAILABLE_THEMES } from '../../stores/theme.store';
import { useLanguage } from '../../i18n/index';

// Theme display names and descriptions
const getThemeInfo = (theme: DaisyUITheme, t: any) => {
  const themeInfoMap: Record<DaisyUITheme, { name: string; description: string; category: string }> = {
    light: { name: t('settings.themeLight'), description: '明亮清新的浅色主题', category: '基础' },
    dark: { name: t('settings.themeDark'), description: '优雅的深色主题', category: '基础' },
    cupcake: { name: 'Cupcake', description: '温馨的粉色主题', category: '彩色' },
    bumblebee: { name: 'Bumblebee', description: '活力的黄色主题', category: '彩色' },
    emerald: { name: 'Emerald', description: '清新的绿色主题', category: '彩色' },
    corporate: { name: 'Corporate', description: '专业的商务主题', category: '商务' },
    synthwave: { name: 'Synthwave', description: '复古未来主义主题', category: '特色' },
    retro: { name: 'Retro', description: '怀旧复古主题', category: '特色' },
    cyberpunk: { name: 'Cyberpunk', description: '赛博朋克主题', category: '特色' },
    valentine: { name: 'Valentine', description: '浪漫的情人节主题', category: '节日' },
    halloween: { name: 'Halloween', description: '神秘的万圣节主题', category: '节日' },
    garden: { name: 'Garden', description: '自然的花园主题', category: '自然' },
    forest: { name: 'Forest', description: '深邃的森林主题', category: '自然' },
    aqua: { name: 'Aqua', description: '清澈的水蓝主题', category: '自然' },
    lofi: { name: 'Lo-Fi', description: '低保真美学主题', category: '艺术' },
    pastel: { name: 'Pastel', description: '柔和的马卡龙主题', category: '艺术' },
    fantasy: { name: 'Fantasy', description: '梦幻的奇幻主题', category: '艺术' },
    wireframe: { name: 'Wireframe', description: '极简的线框主题', category: '极简' },
    black: { name: 'Black', description: '纯黑极简主题', category: '极简' },
    luxury: { name: 'Luxury', description: '奢华的金色主题', category: '高端' },
    dracula: { name: 'Dracula', description: '经典的德古拉主题', category: '高端' },
    cmyk: { name: 'CMYK', description: '印刷色彩主题', category: '专业' },
    autumn: { name: 'Autumn', description: '温暖的秋日主题', category: '季节' },
    business: { name: 'Business', description: '严肃的商务主题', category: '商务' },
    acid: { name: 'Acid', description: '酸性荧光主题', category: '特色' },
    lemonade: { name: 'Lemonade', description: '清爽的柠檬主题', category: '彩色' },
    night: { name: 'Night', description: '深邃的夜晚主题', category: '深色' },
    coffee: { name: 'Coffee', description: '温暖的咖啡主题', category: '温暖' },
    winter: { name: 'Winter', description: '清冷的冬日主题', category: '季节' },
    dim: { name: 'Dim', description: '柔和的暗色主题', category: '深色' },
    nord: { name: 'Nord', description: '北欧风格主题', category: '极简' },
    sunset: { name: 'Sunset', description: '温暖的日落主题', category: '温暖' },
  };
  
  return themeInfoMap[theme] || { name: theme, description: '', category: '其他' };
};

// Group themes by category
const groupThemesByCategory = (themes: readonly DaisyUITheme[], t: any) => {
  const grouped: Record<string, DaisyUITheme[]> = {};
  
  themes.forEach(theme => {
    const info = getThemeInfo(theme, t);
    if (!grouped[info.category]) {
      grouped[info.category] = [];
    }
    grouped[info.category].push(theme);
  });
  
  return grouped;
};

export const ThemeSelector = () => {
  const { t } = useLanguage();
  const [isOpen, setIsOpen] = createSignal(false);
  let containerRef: HTMLDivElement | undefined;

  const groupedThemes = () => groupThemesByCategory(AVAILABLE_THEMES, t);

  const handleThemeChange = (theme: DaisyUITheme) => {
    themeStore.setCurrentTheme(theme);
    setIsOpen(false);
  };

  const currentThemeInfo = () => getThemeInfo(themeStore.currentTheme(), t);

  // Close dropdown when clicking outside
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
          onClick={() => setIsOpen(!isOpen())}
        >
          <div class="flex items-center gap-3">
            <div class="flex gap-1">
              <div class="w-3 h-3 rounded-full bg-primary"></div>
              <div class="w-3 h-3 rounded-full bg-secondary"></div>
              <div class="w-3 h-3 rounded-full bg-accent"></div>
            </div>
            <span>{currentThemeInfo().name}</span>
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
          <div class="absolute top-full left-0 right-0 z-[1000] mt-1 menu p-2 shadow-lg bg-base-100 rounded-box max-h-96 overflow-y-auto border border-base-300">
            <For each={Object.entries(groupedThemes())}>
              {([category, themes]) => (
                <div class="mb-2">
                  <div class="menu-title text-xs font-semibold text-base-content/70 px-2 py-1">
                    {category}
                  </div>
                  <For each={themes}>
                    {(theme) => {
                      const info = getThemeInfo(theme, t);
                      return (
                        <li>
                          <button
                            class={`flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 ${
                              themeStore.currentTheme() === theme ? 'bg-primary/10 text-primary' : ''
                            }`}
                            onClick={() => handleThemeChange(theme)}
                          >
                            <div class="flex gap-1 flex-shrink-0">
                              <div class="w-2 h-2 rounded-full bg-primary opacity-80"></div>
                              <div class="w-2 h-2 rounded-full bg-secondary opacity-80"></div>
                              <div class="w-2 h-2 rounded-full bg-accent opacity-80"></div>
                            </div>
                            <div class="flex-1 text-left">
                              <div class="font-medium">{info.name}</div>
                              <div class="text-xs text-base-content/60">{info.description}</div>
                            </div>
                            {themeStore.currentTheme() === theme && (
                              <svg class="w-4 h-4 text-primary" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                              </svg>
                            )}
                          </button>
                        </li>
                      );
                    }}
                  </For>
                </div>
              )}
            </For>
          </div>
        )}
      </div>

      <label class="label">
        <span class="label-text-alt text-base-content/60">{t('settings.themeDescription')}</span>
      </label>
    </div>
  );
};
