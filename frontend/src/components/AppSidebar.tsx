import { useEffect } from 'react';
import type { AppConfig } from '../types/apps';
import { AppIcon } from './AppIcon';

interface AppSidebarProps {
  apps: AppConfig[];
  selectedAppId: string | null;
  onSelectApp: (appId: string | null) => void;
  onLaunchApp?: (appId: string) => void;
  onStopApp?: (appId: string) => void;
  onOpenLog?: (appId: string) => void;
}

/** Fixed launcher for the inference plugins compiled into this build. */
export function AppSidebar({
  apps,
  selectedAppId,
  onSelectApp,
  onLaunchApp,
  onStopApp,
  onOpenLog,
}: AppSidebarProps) {
  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && selectedAppId) {
        onSelectApp(null);
      }
    };

    window.addEventListener('keydown', handleEscape);
    return () => window.removeEventListener('keydown', handleEscape);
  }, [onSelectApp, selectedAppId]);

  return (
    <div
      className="flex h-auto w-16 flex-col items-center gap-3 overflow-visible border-r-0 border-[hsl(var(--launcher-border))] bg-[hsl(var(--launcher-bg-secondary)/0.5)] px-1 py-4 font-mono font-normal shadow-none"
      role="toolbar"
      aria-label="Inference plugins"
    >
      {apps.map((app) => (
        <AppIcon
          key={app.id}
          appId={app.id}
          state={app.iconState}
          isSelected={selectedAppId === app.id}
          onClick={() => onSelectApp(selectedAppId === app.id ? null : app.id)}
          title={app.displayName}
          ramUsage={app.ramUsage}
          gpuUsage={app.gpuUsage}
          hasInstall={app.iconState !== 'uninstalled'}
          launchError={app.iconState === 'error'}
          onLaunch={() => onLaunchApp?.(app.id)}
          onStop={() => onStopApp?.(app.id)}
          onOpenLog={() => onOpenLog?.(app.id)}
        />
      ))}
    </div>
  );
}
