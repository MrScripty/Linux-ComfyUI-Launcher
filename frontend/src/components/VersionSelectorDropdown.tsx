import type { RefObject } from 'react';
import { useHover } from '@react-aria/interactions';
import { Check } from 'lucide-react';
import { VersionSelectorDefaultButton } from './VersionSelectorDefaultButton';

interface VersionDropdownItemProps {
  version: string;
  isActive: boolean;
  isInstalling: boolean;
  isSwitching: boolean;
  isLoading: boolean;
  isDefault: boolean;
  onMakeDefault?: (tag: string | null) => Promise<boolean>;
  onSwitchVersion: (tag: string) => void;
  switchButtonRef?: RefObject<HTMLButtonElement | null>;
}

function VersionDropdownItem({
  version,
  isActive,
  isInstalling,
  isSwitching,
  isLoading,
  isDefault,
  onMakeDefault,
  onSwitchVersion,
  switchButtonRef,
}: VersionDropdownItemProps) {
  const { hoverProps: rowHoverProps, isHovered: isRowHovered } = useHover({});

  return (
    <div
      {...rowHoverProps}
      className={`relative flex w-full items-center justify-between px-3 py-2 text-left text-sm transition-colors ${
        isActive
          ? 'bg-[hsl(var(--surface-interactive-hover))] text-[hsl(var(--accent-success))]'
          : isInstalling
            ? 'bg-[hsl(var(--surface-interactive))] text-[hsl(var(--text-tertiary))]'
            : 'text-[hsl(var(--text-secondary))] hover:bg-[hsl(var(--surface-interactive-hover))] hover:text-[hsl(var(--text-primary))]'
      } ${isSwitching || isInstalling ? 'cursor-not-allowed opacity-50' : ''}`}
    >
      <div className="flex min-w-0 items-center gap-2">
        <div className="flex w-4 flex-shrink-0 items-center justify-center">
          {onMakeDefault ? (
            <VersionSelectorDefaultButton
              isDefault={isDefault}
              isLoading={isLoading}
              isRowHovered={isRowHovered}
              isSwitching={isSwitching}
              onMakeDefault={onMakeDefault}
              version={version}
            />
          ) : (
            <div className="w-4" />
          )}
        </div>
        <button
          ref={switchButtonRef}
          type="button"
          onClick={() => onSwitchVersion(version)}
          disabled={isInstalling}
          className="flex min-w-0 flex-1 items-center gap-2 bg-transparent p-0 text-left disabled:cursor-not-allowed"
          aria-label={`Switch to ${version}`}
        >
          <span className="truncate font-medium">{version}</span>
          {isInstalling && (
            <span className="rounded-full border border-amber-400/60 bg-amber-500/20 px-1.5 py-[2px] text-[10px] text-amber-200">
              Installing
            </span>
          )}
        </button>
      </div>
      <div className="flex items-center gap-2 pr-12">
        {isActive && (
          <span className="absolute right-2 top-1/2 -translate-y-1/2">
            <Check size={14} className="text-[hsl(var(--accent-success))]" />
          </span>
        )}
      </div>
    </div>
  );
}

interface VersionSelectorDropdownProps {
  combinedVersions: string[];
  activeVersion: string | null;
  installingVersion: string | null | undefined;
  installedVersions: string[];
  isInstallComplete: boolean;
  defaultVersion: string | null;
  isSwitching: boolean;
  isLoading: boolean;
  initialFocusRef: RefObject<HTMLButtonElement | null>;
  onMakeDefault?: (tag: string | null) => Promise<boolean>;
  onSwitchVersion: (tag: string) => void;
}

export function VersionSelectorDropdown({
  combinedVersions,
  activeVersion,
  installingVersion,
  installedVersions,
  isInstallComplete,
  defaultVersion,
  isSwitching,
  isLoading,
  initialFocusRef,
  onMakeDefault,
  onSwitchVersion,
}: VersionSelectorDropdownProps) {
  return (
    <>
      <div className="max-h-64 overflow-y-auto">
        {combinedVersions.map((version, index) => {
          const isActive = version === activeVersion;
          const isInstalling =
            installingVersion === version &&
            !installedVersions.includes(version) &&
            !isInstallComplete;
          const isDefault = defaultVersion === version;
          const receivesInitialFocus =
            !isInstalling && (isActive || (!activeVersion && index === 0));
          return (
            <VersionDropdownItem
              key={version}
              version={version}
              isActive={isActive}
              isInstalling={isInstalling}
              isSwitching={isSwitching}
              isLoading={isLoading}
              isDefault={isDefault}
              onMakeDefault={onMakeDefault}
              onSwitchVersion={onSwitchVersion}
              switchButtonRef={receivesInitialFocus ? initialFocusRef : undefined}
            />
          );
        })}
      </div>

      {installedVersions.length === 0 && (
        <div className="px-3 py-4 text-center text-sm text-[hsl(var(--text-tertiary))]">
          No versions installed
        </div>
      )}
    </>
  );
}
