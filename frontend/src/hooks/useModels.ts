/**
 * Models management hook
 *
 * Handles model fetching, scanning, organization, and FTS search.
 * Implements Stale-While-Revalidate (SWR) pattern for instant UI response.
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { isAPIAvailable } from '../api/adapter';
import { modelsAPI } from '../api/models';
import { importAPI } from '../api/import';
import type { ModelCategory } from '../types/apps';
import type { CatalogModel } from '../generated/desktop-contract';
import { getLogger } from '../utils/logger';
import { APIError } from '../errors';
import { groupCatalogModels } from '../utils/libraryModels';
import {
  readModelLibrarySnapshot,
  toDisplayOnlyModelGroups,
  writeModelLibrarySnapshot,
} from '../utils/modelLibrarySnapshot';
import { useModelLibraryUpdateSubscription } from './useModelLibraryUpdateSubscription';
import { useLibraryScopeId } from './useLauncherRootRecovery';

const logger = getLogger('useModels');

/** Debounce delay for search queries (ms) */
const SEARCH_DEBOUNCE_MS = 300;

/** Cache TTL for SWR pattern (ms) - show cached results for up to 30 seconds */
const CACHE_TTL_MS = 30000;

export type ModelLibraryLoadStatus = 'loading' | 'ready' | 'unavailable';

/** Cache entry for SWR pattern */
interface CacheEntry {
  query: string;
  results: ModelCategory[];
  queryTime: number | null;
  timestamp: number;
}

export function useModels() {
  const libraryScopeId = useLibraryScopeId();
  const [modelGroups, setModelGroups] = useState<ModelCategory[]>(() => readModelLibrarySnapshot(libraryScopeId));
  const [libraryLoadStatus, setLibraryLoadStatus] = useState<ModelLibraryLoadStatus>('loading');
  const [isSearching, setIsSearching] = useState(false);
  const [isRevalidating, setIsRevalidating] = useState(false);
  const [searchQueryTime, setSearchQueryTime] = useState<number | null>(null);
  const [hasNewResults, setHasNewResults] = useState(false);
  const searchSequenceRef = useRef(0);
  const fetchSequenceRef = useRef(0);
  const searchTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const searchCacheRef = useRef<Map<string, CacheEntry>>(new Map());
  const activeSearchRef = useRef<{
    query: string;
  } | null>(null);

  const fetchModels = useCallback(async () => {
    const currentSequence = ++fetchSequenceRef.current;
    setModelGroups(toDisplayOnlyModelGroups);
    // Check API availability before fetching
    if (!isAPIAvailable()) {
      logger.debug('API not available yet, skipping fetch');
      setLibraryLoadStatus('unavailable');
      return;
    }

    setLibraryLoadStatus('loading');

    try {
      const result = await modelsAPI.getModels();
      if (currentSequence === fetchSequenceRef.current) {
        setLibraryLoadStatus('ready');
      }
      if (currentSequence !== fetchSequenceRef.current || activeSearchRef.current) {
        logger.debug('Discarding stale model list response', {
          currentSequence,
          latestSequence: fetchSequenceRef.current,
          activeSearch: Boolean(activeSearchRef.current),
        });
        return;
      }

      const freshModelGroups = groupCatalogModels(Object.values(result.models));
      setModelGroups(freshModelGroups);
      if (!writeModelLibrarySnapshot(freshModelGroups, libraryScopeId)) {
        logger.debug('Model library startup snapshot could not be persisted');
      }
    } catch (error) {
      if (currentSequence === fetchSequenceRef.current) {
        setLibraryLoadStatus('unavailable');
      }
      if (error instanceof APIError) {
        logger.error('API error fetching models', { error: error.message, endpoint: error.endpoint });
      } else if (error instanceof Error) {
        logger.error('Unexpected error fetching models', { error: error.message });
      } else {
        logger.error('Unknown error fetching models', { error });
      }
    }
  }, [libraryScopeId]);

  const scanModels = useCallback(async () => {
    try {
      const result = await modelsAPI.scanSharedStorage();
      if (result.success) {
        await fetchModels();
      }
    } catch (error) {
      if (error instanceof APIError) {
        logger.error('API error scanning models', { error: error.message, endpoint: error.endpoint });
      } else if (error instanceof Error) {
        logger.error('Unexpected error scanning models', { error: error.message });
      } else {
        logger.error('Unknown error scanning models', { error });
      }
    }
  }, [fetchModels]);

  // Note: Polling removed - file watching will be implemented in the backend
  // to notify the frontend when models change, instead of polling every 10 seconds

  // Initial fetch
  useEffect(() => {
    void fetchModels();
  }, [fetchModels]);

  /**
   * Transform FTS results to ModelCategory format
   */
  const transformFTSResults = useCallback(
    (models: readonly CatalogModel[]): ModelCategory[] => {
      return groupCatalogModels(models);
    },
    []
  );

  /**
   * Dismiss the "new results available" notification
   */
  const dismissNewResults = useCallback(() => {
    setHasNewResults(false);
  }, []);

  /**
   * Debounced FTS search for models.
   * Implements Stale-While-Revalidate (SWR) pattern:
   * 1. Show cached results immediately if available
   * 2. Fetch fresh results in background
   * 3. Update UI when new results arrive
   *
   * Uses sequence guards to discard stale responses.
   */
  const searchModelsFTS = useCallback(
    (query: string) => {
      // Clear any pending search
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }

      // Increment sequence for every search invocation, including resets,
      // so in-flight responses from older queries cannot overwrite newer UI state.
      const currentSequence = ++searchSequenceRef.current;

      // Reset new results notification
      setHasNewResults(false);

      // Empty query - reset to full list
      if (!query.trim()) {
        activeSearchRef.current = null;
        setIsSearching(false);
        setIsRevalidating(false);
        setSearchQueryTime(null);
        void fetchModels();
        return;
      }

      activeSearchRef.current = { query };

      // Check cache for immediate response (SWR pattern)
      const cacheKey = query;
      const cached = searchCacheRef.current.get(cacheKey);
      const now = Date.now();

      if (cached && now - cached.timestamp < CACHE_TTL_MS) {
        // Show cached results immediately
        setModelGroups(toDisplayOnlyModelGroups(cached.results));
        setSearchQueryTime(cached.queryTime);
        setIsSearching(false);
        setIsRevalidating(true);
        logger.debug('Showing cached results for query', { query, age: now - cached.timestamp });
      } else {
        setIsSearching(true);
        setIsRevalidating(false);
      }

      // Debounce the search (revalidation happens in background)
      searchTimeoutRef.current = setTimeout(async () => {
        try {
          const result = await importAPI.searchModelsFTS(query, 100, 0);

          // Sequence guard: discard stale responses
          if (currentSequence !== searchSequenceRef.current) {
            logger.debug('Discarding stale search response', {
              currentSequence,
              latestSequence: searchSequenceRef.current,
            });
            return;
          }

          const categorizedModels = transformFTSResults(result.models);
          const displayResults = toDisplayOnlyModelGroups(categorizedModels);

          const resultsChanged =
            cached && JSON.stringify(displayResults) !== JSON.stringify(cached.results);

          searchCacheRef.current.set(cacheKey, {
            query,
            results: displayResults,
            queryTime: result.query_time_ms,
            timestamp: Date.now(),
          });

          setModelGroups(categorizedModels);
          setSearchQueryTime(result.query_time_ms);

          if (resultsChanged) {
            setHasNewResults(true);
            logger.debug('New results available after revalidation', { query });
          }
        } catch (error) {
          if (error instanceof APIError) {
            logger.error('API error in FTS search', {
              error: error.message,
              endpoint: error.endpoint,
            });
          } else if (error instanceof Error) {
            logger.error('Unexpected error in FTS search', { error: error.message });
          } else {
            logger.error('Unknown error in FTS search', { error });
          }
        } finally {
          if (currentSequence === searchSequenceRef.current) {
            setIsSearching(false);
            setIsRevalidating(false);
          }
        }
      }, SEARCH_DEBOUNCE_MS);
    },
    [fetchModels, transformFTSResults]
  );

  useModelLibraryUpdateSubscription(
    useCallback(() => {
      searchCacheRef.current.clear();

      const activeSearch = activeSearchRef.current;
      if (activeSearch?.query.trim()) {
        searchModelsFTS(activeSearch.query);
        return;
      }

      void fetchModels();
    }, [fetchModels, searchModelsFTS])
  );

  // Cleanup search timeout on unmount
  useEffect(() => {
    return () => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }
    };
  }, []);

  return {
    modelGroups,
    libraryLoadStatus,
    fetchModels,
    scanModels,
    searchModelsFTS,
    isSearching,
    isRevalidating,
    searchQueryTime,
    hasNewResults,
    dismissNewResults,
  };
}
