'use client';

import React from 'react';
import { X, Info, Shield } from 'lucide-react';
import { useI18n } from '@/i18n';

interface AnalyticsDataModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirmDisable: () => void;
}

export default function AnalyticsDataModal({ isOpen, onClose, onConfirmDisable }: AnalyticsDataModalProps) {
  const { t } = useI18n();
  if (!isOpen) return null;

  const categories = [
    ['analyticsModal.modelsTitle', 'analyticsModal.modelsItems', 'analyticsModal.modelsReason'],
    ['analyticsModal.metricsTitle', 'analyticsModal.metricsItems', 'analyticsModal.metricsReason'],
    ['analyticsModal.devicesTitle', 'analyticsModal.devicesItems', 'analyticsModal.devicesReason'],
    ['analyticsModal.usageTitle', 'analyticsModal.usageItems', 'analyticsModal.usageReason'],
    ['analyticsModal.platformTitle', 'analyticsModal.platformItems', 'analyticsModal.platformReason'],
  ] as const;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <div className="flex items-center gap-3">
            <Shield className="w-6 h-6 text-blue-600" />
            <h2 className="text-xl font-semibold text-gray-900">{t('analyticsModal.title')}</h2>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Privacy Notice */}
          <div className="bg-green-50 border border-green-200 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <Info className="w-5 h-5 text-green-600 mt-0.5 flex-shrink-0" />
              <div className="text-sm text-green-800">
                <p className="font-semibold mb-1">{t('analyticsModal.privacyTitle')}</p>
                <p>{t('analyticsModal.privacyBody')}</p>
              </div>
            </div>
          </div>

          {/* Data Categories */}
          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-gray-900">{t('analyticsModal.collectedTitle')}</h3>
            {categories.map(([titleKey, itemsKey, reasonKey]) => (
              <div key={titleKey} className="border border-gray-200 rounded-lg p-4">
                <h4 className="font-semibold text-gray-900 mb-2">{t(titleKey)}</h4>
                <ul className="text-sm text-gray-700 space-y-1 ml-4">
                  {t(itemsKey).split('|').map((item) => <li key={item}>• {item}</li>)}
                </ul>
                <p className="text-xs text-gray-500 mt-2 italic">{t(reasonKey)}</p>
              </div>
            ))}
          </div>

          {/* What We DON'T Collect */}
          <div className="bg-red-50 border border-red-200 rounded-lg p-4">
            <h4 className="font-semibold text-red-900 mb-2">{t('analyticsModal.notCollectedTitle')}</h4>
            <ul className="text-sm text-red-800 space-y-1 ml-4">
              {t('analyticsModal.notCollectedItems').split('|').map((item) => <li key={item}>• ❌ {item}</li>)}
            </ul>
          </div>

          {/* Example Event */}
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
            <h4 className="font-semibold text-gray-900 mb-2">{t('analyticsModal.example')}</h4>
            <pre className="text-xs text-gray-700 overflow-x-auto">
              {`{
  "event": "meeting_ended",
  "app_version": "0.4.1",
  "transcription_provider": "parakeet",
  "transcription_model": "parakeet-tdt-0.6b-v3-int8",
  "summary_provider": "ollama",
  "summary_model": "llama3.2:latest",
  "total_duration_seconds": "125.5",
  "microphone_device_type": "Wired",
  "system_audio_device_type": "Bluetooth",
  "chunks_processed": "150",
  "had_fatal_error": "false"
}`}
            </pre>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between gap-4 p-6 border-t border-gray-200 bg-gray-50">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
          >
            {t('analyticsModal.keepEnabled')}
          </button>
          <button
            onClick={onConfirmDisable}
            className="px-4 py-2 text-white bg-red-600 rounded-md hover:bg-red-700 transition-colors"
          >
            {t('analyticsModal.confirmDisable')}
          </button>
        </div>
      </div>
    </div>
  );
}
