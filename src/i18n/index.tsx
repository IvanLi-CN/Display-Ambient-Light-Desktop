import { createSignal, createContext, useContext, ParentComponent, createEffect } from 'solid-js';
import { Language, TranslationDict } from './types';
import { zhCN } from './locales/zh-CN';
import { enUS } from './locales/en-US';

// Available translations
const translations: Record<Language, TranslationDict> = {
  'zh-CN': zhCN,
  'en-US': enUS,
};

// Create locale signal
const [locale, setLocale] = createSignal<Language>('zh-CN');

// Translation function
const t = (key: string): string => {
  const keys = key.split('.');
  let value: any = translations[locale()];

  for (const k of keys) {
    if (value && typeof value === 'object' && k in value) {
      value = value[k];
    } else {
      return key; // Return key if translation not found
    }
  }

  return typeof value === 'string' ? value : key;
};

// Language context for managing language state
interface LanguageContextType {
  locale: () => Language;
  setLocale: (lang: Language) => void;
  t: (key: string) => string;
}

const LanguageContext = createContext<LanguageContextType>();

// Language provider component
export const LanguageProvider: ParentComponent = (props) => {
  // Load saved language preference from localStorage
  createEffect(() => {
    const savedLang = localStorage.getItem('app-language') as Language;
    if (savedLang && (savedLang === 'zh-CN' || savedLang === 'en-US')) {
      setLocale(savedLang);
    }
  });

  // Save language preference when it changes
  createEffect(() => {
    localStorage.setItem('app-language', locale());
  });

  const contextValue: LanguageContextType = {
    locale,
    setLocale,
    t,
  };

  return (
    <LanguageContext.Provider value={contextValue}>
      {props.children}
    </LanguageContext.Provider>
  );
};

// Hook to use language context
export const useLanguage = () => {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error('useLanguage must be used within a LanguageProvider');
  }
  return context;
};

// Export types and utilities
export type { Language, TranslationDict };
export { translations };
