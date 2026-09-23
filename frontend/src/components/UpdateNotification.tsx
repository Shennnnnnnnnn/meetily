import React from 'react';
import { Download } from 'lucide-react';
import { toast } from 'sonner';
import { UpdateInfo } from '@/services/updateService';
import { currentLocale, translate } from '@/i18n';

let globalShowDialogCallback: (() => void) | null = null;

export function setUpdateDialogCallback(callback: () => void) {
  globalShowDialogCallback = callback;
}

export function showUpdateNotification(updateInfo: UpdateInfo, onUpdateClick?: () => void) {
  const handleClick = () => {
    if (onUpdateClick) {
      onUpdateClick();
    } else if (globalShowDialogCallback) {
      globalShowDialogCallback();
    }
  };

  toast.info(
    <div className="flex items-center justify-between gap-4">
      <div className="flex items-center gap-2">
        <Download className="h-4 w-4" />
        <div>
          <p className="font-medium">{translate(currentLocale(), 'update.available')}</p>
          <p className="text-sm text-muted-foreground">
            {translate(currentLocale(), 'update.versionAvailable', { version: updateInfo.version ?? '' })}
          </p>
        </div>
      </div>
      <button
        onClick={(e) => {
          e.stopPropagation();
          handleClick();
        }}
        className="text-sm font-medium text-blue-600 hover:text-blue-700 underline"
      >
        {translate(currentLocale(), 'update.viewDetails')}
      </button>
    </div>,
    {
      duration: 10000,
      position: 'bottom-center',
    }
  );
}
