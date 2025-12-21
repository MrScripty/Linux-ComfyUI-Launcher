import React, { useCallback, useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Check, Loader2, Download, FolderOpen, CheckCircle2 } from 'lucide-react';

interface VersionSelectorProps {
  installedVersions: string[];
  activeVersion: string | null;
  isLoading: boolean;
  switchVersion: (tag: string) => Promise<boolean>;
  openActiveInstall: () => Promise<boolean>;
  onOpenVersionManager: () => void;
  activeShortcutState?: { menu: boolean; desktop: boolean };
}

export function VersionSelector({
  installedVersions,
  activeVersion,
  isLoading,
  switchVersion,
  openActiveInstall,
  onOpenVersionManager,
  activeShortcutState,
}: VersionSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isSwitching, setIsSwitching] = useState(false);
  const [isOpeningPath, setIsOpeningPath] = useState(false);
  const [showOpenedIndicator, setShowOpenedIndicator] = useState(false);
  const openedIndicatorTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [hoveredVersion, setHoveredVersion] = useState<string | null>(null);
  const [shortcutState, setShortcutState] = useState<Record<string, { menu: boolean; desktop: boolean }>>({});

  const refreshShortcutStates = useCallback(async () => {
    if (!window.pywebview?.api?.get_all_shortcut_states) {
      return;
    }

    try {
      const result = await window.pywebview.api.get_all_shortcut_states();
      const states = result?.states?.states;
      if (result.success && states) {
        const mapped: Record<string, { menu: boolean; desktop: boolean }> = {};
        Object.entries(states).forEach(([tag, state]) => {
          mapped[tag] = {
            menu: Boolean((state as any).menu),
            desktop: Boolean((state as any).desktop),
          };
        });
        setShortcutState(mapped);
      }
    } catch (err) {
      console.error('Failed to fetch shortcut states', err);
    }
  }, []);

  const handleVersionSwitch = async (tag: string) => {
    if (tag === activeVersion) {
      setIsOpen(false);
      return;
    }

    setIsSwitching(true);
    try {
      await switchVersion(tag);
      setIsOpen(false);
    } catch (e) {
      console.error('Failed to switch version:', e);
    } finally {
      setIsSwitching(false);
    }
  };

  const handleOpenActiveInstall = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!activeVersion) {
      return;
    }

    setIsOpeningPath(true);
    setShowOpenedIndicator(false);
    try {
      await openActiveInstall();
      setShowOpenedIndicator(true);
      if (openedIndicatorTimeout.current) {
        clearTimeout(openedIndicatorTimeout.current);
      }
      openedIndicatorTimeout.current = setTimeout(() => setShowOpenedIndicator(false), 2000);
    } catch (err) {
      console.error('Failed to open active installation path:', err);
    } finally {
      setIsOpeningPath(false);
    }
  };

  const displayVersion = activeVersion || 'No version selected';
  const hasInstalledVersions = installedVersions.length > 0;
  const emphasizeInstall = !hasInstalledVersions && !isLoading;

  useEffect(() => {
    return () => {
      if (openedIndicatorTimeout.current) {
        clearTimeout(openedIndicatorTimeout.current);
      }
    };
  }, []);

  // Refresh shortcut states when installed versions change
  useEffect(() => {
    if (!installedVersions.length) {
      setShortcutState({});
      return;
    }
    refreshShortcutStates();
  }, [installedVersions, refreshShortcutStates]);

  // Keep active version shortcut in sync with main toggle state
  useEffect(() => {
    if (activeVersion && activeShortcutState) {
      setShortcutState((prev) => ({
        ...prev,
        [activeVersion]: {
          menu: activeShortcutState.menu,
          desktop: activeShortcutState.desktop,
        },
      }));
    }
  }, [activeVersion, activeShortcutState?.menu, activeShortcutState?.desktop]);

  return (
    <div className="relative w-full">
      {/* Version Selector Container - Changed from button to div to allow nested buttons */}
      <div
        className={`w-full h-10 bg-[#2a2a2a] border border-[#444] rounded flex items-center justify-between px-3 transition-colors ${
          !hasInstalledVersions || isLoading || isSwitching
            ? 'opacity-50'
            : ''
        }`}
      >
        {/* Left side - clickable area for version selector */}
        <button
          onClick={() => {
            console.log('Version selector clicked, hasInstalledVersions:', hasInstalledVersions, 'isOpen:', isOpen);
            if (hasInstalledVersions) {
              setIsOpen(!isOpen);
              console.log('Set isOpen to:', !isOpen);
            }
          }}
          disabled={!hasInstalledVersions || isLoading || isSwitching}
          className="flex items-center gap-2 flex-1 hover:opacity-80 transition-opacity disabled:cursor-not-allowed"
        >
          {isSwitching ? (
            <Loader2 size={14} className="text-gray-400 animate-spin" />
          ) : (
            <div className="w-2 h-2 rounded-full bg-[#55ff55]" />
          )}
          <span className="text-sm text-white font-medium">
            {isSwitching ? 'Switching...' : displayVersion}
          </span>
        </button>

        {/* Right side - action buttons */}
        <div className="flex items-center gap-2">
          {/* Open in File Manager */}
          <motion.button
            onClick={handleOpenActiveInstall}
            disabled={!activeVersion || isOpeningPath || isLoading}
            className="p-1 rounded hover:bg-[#444] transition-colors disabled:opacity-50"
            whileHover={{ scale: activeVersion ? 1.1 : 1 }}
            whileTap={{ scale: activeVersion ? 0.9 : 1 }}
            title={activeVersion ? 'Open active version in file manager' : 'No active version to open'}
          >
            {showOpenedIndicator ? (
              <CheckCircle2 size={14} className="text-[#55ff55]" />
            ) : isOpeningPath ? (
              <Loader2 size={14} className="text-gray-400 animate-spin" />
            ) : (
              <FolderOpen size={14} className="text-gray-400" />
            )}
          </motion.button>

          {/* Download Button */}
          <motion.button
            onClick={(e) => {
              console.log('Download button clicked!');
              e.stopPropagation();
              onOpenVersionManager();
              console.log('Switching to version manager view');
            }}
            disabled={isLoading}
            className={`p-1 rounded hover:bg-[#444] transition-colors disabled:opacity-50 ${
              emphasizeInstall ? 'animate-pulse ring-1 ring-[#55ff55]/60' : ''
            }`}
            whileHover={{ scale: 1.1 }}
            whileTap={{ scale: 0.9 }}
            title="Install new version"
          >
            <Download size={14} className="text-gray-400" />
          </motion.button>

          {/* Refresh Button moved to manager; dropdown arrow removed for cleaner look */}
        </div>
      </div>

      {/* Dropdown Menu */}
      <AnimatePresence>
        {isOpen && hasInstalledVersions && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            className="absolute top-full left-0 right-0 mt-1 bg-[#2a2a2a] border border-[#444] rounded shadow-lg overflow-hidden z-50"
          >
            <div className="max-h-64 overflow-y-auto">
              {installedVersions.map((version) => {
                const isActive = version === activeVersion;
                const toggles = shortcutState[version] || { menu: false, desktop: false };
                const isEnabled = toggles.menu && toggles.desktop;
                const isPartial = toggles.menu !== toggles.desktop;
                const showHover = hoveredVersion === version;
                return (
                  <div
                    key={version}
                    onClick={() => handleVersionSwitch(version)}
                    onMouseEnter={() => setHoveredVersion(version)}
                    onMouseLeave={() => setHoveredVersion((prev) => (prev === version ? null : prev))}
                    className={`relative w-full px-3 py-2 text-left text-sm flex items-center justify-between transition-colors ${
                      isActive
                        ? 'bg-[#333333] text-[#55ff55]'
                        : 'text-gray-300 hover:bg-[#333333] hover:text-white'
                    } ${isSwitching ? 'opacity-50 cursor-not-allowed' : ''}`}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="font-medium truncate">{version}</span>
                      {isActive && (
                        <span className="px-1.5 py-[2px] text-[10px] rounded-full bg-[#2a2a2a] border border-[#55ff55]/60 text-[#55ff55]">
                          Default
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-2 pr-12">
                      <div
                        className={`absolute right-8 top-1/2 -translate-y-1/2 transition-opacity duration-150 ${showHover ? 'opacity-100' : 'opacity-0 pointer-events-none'}`}
                        onClick={(e) => e.stopPropagation()}
                      >
                        <button
                          onClick={async () => {
                            if (!window.pywebview?.api?.set_version_shortcuts) {
                              console.warn('Shortcut API not available');
                              return;
                            }

                            const next = !isEnabled;
                            setShortcutState((prev) => ({
                              ...prev,
                              [version]: { menu: next, desktop: next },
                            }));

                            try {
                              if (window.pywebview?.api?.set_version_shortcuts) {
                                const result = await window.pywebview.api.set_version_shortcuts(version, next);
                                if (result?.state) {
                                  setShortcutState((prev) => ({
                                    ...prev,
                                    [version]: {
                                      menu: Boolean(result.state.menu),
                                      desktop: Boolean(result.state.desktop),
                                    },
                                  }));
                                }
                              }
                            } catch (err) {
                              console.error('Failed to toggle shortcuts', err);
                              // revert on failure
                              setShortcutState((prev) => ({
                                ...prev,
                                [version]: { menu: isEnabled, desktop: isEnabled },
                              }));
                            }
                          }}
                          disabled={isSwitching || isLoading}
                          className={`relative w-8 h-4 rounded-full border transition-colors align-middle ${
                            isEnabled
                              ? 'bg-[#55ff55]/30 border-[#55ff55]/70'
                              : isPartial
                                ? 'bg-[#4a3c2a] border-amber-400/80'
                                : 'bg-[#2f2f2f] border-[#555]'
                          } ${isSwitching || isLoading ? 'opacity-50 cursor-not-allowed' : ''}`}
                          title={isEnabled ? 'Remove menu + desktop shortcuts for this version' : 'Create menu + desktop shortcuts for this version'}
                        >
                          <span
                            className={`absolute top-[2px] left-[2px] w-3 h-3 rounded-full bg-white transition-transform ${
                              isEnabled ? 'translate-x-4 bg-[#55ff55]' : 'translate-x-0'
                            }`}
                          />
                        </button>
                      </div>
                      {isActive && (
                        <span className="absolute right-2 top-1/2 -translate-y-1/2">
                          <Check size={14} className="text-[#55ff55]" />
                        </span>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>

            {installedVersions.length === 0 && (
              <div className="px-3 py-4 text-sm text-gray-500 text-center">
                No versions installed
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Empty state hint */}
      {!hasInstalledVersions && (
        <div className="mt-2 text-xs text-gray-400">
          No versions installed yet. Click the download arrow to install your first version.
        </div>
      )}
    </div>
  );
}
