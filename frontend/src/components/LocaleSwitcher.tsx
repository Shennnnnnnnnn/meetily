'use client';

import { Languages } from 'lucide-react';
import { useI18n, type Locale } from '@/i18n';

export function LocaleSwitcher() {
  const { locale, setLocale, t } = useI18n();

  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex items-center gap-2">
        <Languages className="h-4 w-4 text-gray-500" aria-hidden="true" />
        <div>
          <label htmlFor="app-locale" className="text-sm font-medium text-gray-900">
            {t('settings.language')}
          </label>
          <p className="text-xs text-gray-500">{t('settings.languageDescription')}</p>
        </div>
      </div>
      <select
        id="app-locale"
        value={locale}
        onChange={(event) => setLocale(event.target.value as Locale)}
        className="min-w-32 rounded-md border border-gray-300 bg-white px-3 py-2 text-sm text-gray-800 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
      >
        <option value="zh-CN">{t('settings.chineseSimplified')}</option>
        <option value="en-US">{t('settings.englishUs')}</option>
      </select>
    </div>
  );
}

