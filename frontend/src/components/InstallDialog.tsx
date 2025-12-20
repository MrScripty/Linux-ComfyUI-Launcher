import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Download, Check, AlertCircle, Loader2, Calendar, Tag } from 'lucide-react';
import { VersionRelease } from '../hooks/useVersions';

interface InstallDialogProps {
  isOpen: boolean;
  onClose: () => void;
  availableVersions: VersionRelease[];
  installedVersions: string[];
  isLoading: boolean;
  onInstallVersion: (tag: string) => Promise<boolean>;
  onRefreshAll: (forceRefresh: boolean) => Promise<void>;
}

export function InstallDialog({
  isOpen,
  onClose,
  availableVersions,
  installedVersions,
  isLoading,
  onInstallVersion,
  onRefreshAll
}: InstallDialogProps) {
  const [showPreReleases, setShowPreReleases] = useState(true);
  const [showInstalled, setShowInstalled] = useState(true);
  const [installingVersion, setInstallingVersion] = useState<string | null>(null);
  const [installProgress, setInstallProgress] = useState<string | null>(null);
  const [errorVersion, setErrorVersion] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Filter versions based on user preferences
  const filteredVersions = availableVersions.filter((release) => {
    // Filter pre-releases
    if (!showPreReleases && release.prerelease) {
      return false;
    }

    // Filter installed versions
    if (!showInstalled && installedVersions.includes(release.tag_name)) {
      return false;
    }

    return true;
  });

  const handleInstall = async (tag: string) => {
    setInstallingVersion(tag);
    setInstallProgress('Preparing installation...');
    setErrorVersion(null);
    setErrorMessage(null);

    try {
      // Note: The backend doesn't support progress callbacks via PyWebView
      // In a real implementation, we'd poll get_version_status() for progress
      setInstallProgress('Downloading and installing...');

      await onInstallVersion(tag);

      setInstallProgress('Installation complete!');

      // Reset after 2 seconds
      setTimeout(() => {
        setInstallingVersion(null);
        setInstallProgress(null);
      }, 2000);

    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setErrorVersion(tag);
      setErrorMessage(message);
      setInstallProgress(null);
      setInstallingVersion(null);
    }
  };

  const isInstalled = (tag: string) => installedVersions.includes(tag);

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  };

  // Close on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    if (isOpen) {
      window.addEventListener('keydown', handleEscape);
      return () => window.removeEventListener('keydown', handleEscape);
    }
  }, [isOpen, onClose]);

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 bg-black/70 z-50"
          />

          {/* Dialog */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 z-50 flex items-center justify-center p-4"
          >
            <div
              className="bg-[#2a2a2a] border border-[#444] rounded-lg shadow-2xl w-full max-w-3xl max-h-[80vh] flex flex-col"
              onClick={(e) => e.stopPropagation()}
            >
              {/* Header */}
              <div className="flex items-center justify-between p-4 border-b border-[#444]">
                <h2 className="text-xl font-semibold text-white">
                  Install ComfyUI Version
                </h2>
                <button
                  onClick={onClose}
                  className="p-1 rounded hover:bg-[#444] transition-colors"
                >
                  <X size={20} className="text-gray-400" />
                </button>
              </div>

              {/* Filters */}
              <div className="flex items-center gap-4 p-4 border-b border-[#444]">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={showPreReleases}
                    onChange={(e) => setShowPreReleases(e.target.checked)}
                    className="w-4 h-4 rounded border-gray-600 bg-[#333] text-[#55ff55] focus:ring-[#55ff55]"
                  />
                  <span className="text-sm text-gray-300">Show pre-releases</span>
                </label>

                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={showInstalled}
                    onChange={(e) => setShowInstalled(e.target.checked)}
                    className="w-4 h-4 rounded border-gray-600 bg-[#333] text-[#55ff55] focus:ring-[#55ff55]"
                  />
                  <span className="text-sm text-gray-300">Show installed</span>
                </label>

                <div className="flex-1" />

                <span className="text-sm text-gray-500">
                  {filteredVersions.length} version{filteredVersions.length !== 1 ? 's' : ''}
                </span>
              </div>

              {/* Version List */}
              <div className="flex-1 overflow-y-auto p-4">
                {isLoading ? (
                  <div className="flex items-center justify-center py-12">
                    <Loader2 size={32} className="text-gray-400 animate-spin" />
                  </div>
                ) : filteredVersions.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-12 text-gray-500">
                    <AlertCircle size={48} className="mb-3" />
                    <p>No versions available</p>
                    <p className="text-sm mt-1">Try adjusting the filters above</p>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {filteredVersions.map((release) => {
                      const installed = isInstalled(release.tag_name);
                      const installing = installingVersion === release.tag_name;
                      const hasError = errorVersion === release.tag_name;

                      return (
                        <motion.div
                          key={release.tag_name}
                          initial={{ opacity: 0, y: 10 }}
                          animate={{ opacity: 1, y: 0 }}
                          className={`bg-[#333] border rounded-lg p-4 transition-colors ${
                            installed
                              ? 'border-[#55ff55]/30'
                              : hasError
                              ? 'border-red-500/50'
                              : 'border-[#444] hover:border-[#555]'
                          }`}
                        >
                          <div className="flex items-start justify-between gap-4">
                            {/* Version Info */}
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2 mb-1">
                                <Tag size={16} className="text-gray-400 flex-shrink-0" />
                                <h3 className="text-white font-medium truncate">
                                  {release.tag_name}
                                </h3>
                                {release.prerelease && (
                                  <span className="px-2 py-0.5 bg-yellow-500/20 text-yellow-400 text-xs rounded-full">
                                    Pre-release
                                  </span>
                                )}
                                {installed && (
                                  <span className="px-2 py-0.5 bg-[#55ff55]/20 text-[#55ff55] text-xs rounded-full flex items-center gap-1">
                                    <Check size={12} />
                                    Installed
                                  </span>
                                )}
                              </div>

                              <div className="flex items-center gap-2 text-sm text-gray-400 mb-2">
                                <Calendar size={14} />
                                <span>{formatDate(release.published_at)}</span>
                              </div>

                              {release.name && release.name !== release.tag_name && (
                                <p className="text-sm text-gray-300 mb-2">
                                  {release.name}
                                </p>
                              )}

                              {release.body && (
                                <p className="text-sm text-gray-400 line-clamp-2">
                                  {release.body}
                                </p>
                              )}

                              {/* Error message */}
                              {hasError && errorMessage && (
                                <div className="mt-2 flex items-start gap-2 text-sm text-red-400 bg-red-500/10 rounded p-2">
                                  <AlertCircle size={16} className="flex-shrink-0 mt-0.5" />
                                  <span>{errorMessage}</span>
                                </div>
                              )}

                              {/* Progress message */}
                              {installing && installProgress && (
                                <div className="mt-2 flex items-center gap-2 text-sm text-[#55ff55]">
                                  <Loader2 size={16} className="animate-spin" />
                                  <span>{installProgress}</span>
                                </div>
                              )}
                            </div>

                            {/* Install Button */}
                            <motion.button
                              onClick={() => handleInstall(release.tag_name)}
                              disabled={installed || installing || installingVersion !== null}
                              whileHover={!installed && !installing ? { scale: 1.05 } : {}}
                              whileTap={!installed && !installing ? { scale: 0.95 } : {}}
                              className={`flex items-center gap-2 px-4 py-2 rounded font-medium text-sm transition-colors flex-shrink-0 ${
                                installed
                                  ? 'bg-[#55ff55]/20 text-[#55ff55] cursor-not-allowed'
                                  : installing
                                  ? 'bg-gray-700 text-gray-400 cursor-wait'
                                  : installingVersion !== null
                                  ? 'bg-gray-700 text-gray-500 cursor-not-allowed'
                                  : 'bg-[#55ff55] text-black hover:bg-[#66ff66]'
                              }`}
                            >
                              {installed ? (
                                <>
                                  <Check size={16} />
                                  Installed
                                </>
                              ) : installing ? (
                                <>
                                  <Loader2 size={16} className="animate-spin" />
                                  Installing
                                </>
                              ) : (
                                <>
                                  <Download size={16} />
                                  Install
                                </>
                              )}
                            </motion.button>
                          </div>
                        </motion.div>
                      );
                    })}
                  </div>
                )}
              </div>

              {/* Footer */}
              <div className="flex items-center justify-between p-4 border-t border-[#444]">
                <p className="text-sm text-gray-500">
                  {installedVersions.length} version{installedVersions.length !== 1 ? 's' : ''} installed
                </p>
                <button
                  onClick={onClose}
                  className="px-4 py-2 bg-[#444] hover:bg-[#555] text-white rounded font-medium transition-colors"
                >
                  Close
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
