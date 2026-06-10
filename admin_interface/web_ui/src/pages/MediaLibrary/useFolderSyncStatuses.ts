import {useCallback, useEffect, useRef, useState} from 'react';
import {getChildrenSyncStatuses} from '@/util/api';
import {SyncStatus} from '@/types/sync';

const POLL_INTERVAL_MS = 3000;

/**
 * Fetches sync statuses for all direct children of `parentId` in a single
 * request and polls while any entry is pending/running.
 */
export function useFolderSyncStatuses(parentId: number) {
  const [statuses, setStatuses] = useState<Map<number, SyncStatus>>(new Map());
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const refresh = useCallback(async () => {
    const list = await getChildrenSyncStatuses(parentId);
    const map = new Map(list.map(s => [s.libraryEntryId, s]));
    setStatuses(map);
    return list;
  }, [parentId]);

  const stopPolling = useCallback(() => {
    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  const startPolling = useCallback(() => {
    if (intervalRef.current !== null) return; // already polling
    intervalRef.current = setInterval(async () => {
      const list = await getChildrenSyncStatuses(parentId);
      const map = new Map(list.map(s => [s.libraryEntryId, s]));
      setStatuses(map);
      const anyActive = list.some(s => s.status === 'pending' || s.status === 'running');
      if (!anyActive) stopPolling();
    }, POLL_INTERVAL_MS);
  }, [parentId, stopPolling]);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      const list = await getChildrenSyncStatuses(parentId);
      if (cancelled) return;
      setStatuses(new Map(list.map(s => [s.libraryEntryId, s])));
      const anyActive = list.some(s => s.status === 'pending' || s.status === 'running');
      if (anyActive) startPolling();
    };

    load();

    return () => {
      cancelled = true;
      stopPolling();
    };
  }, [parentId, startPolling, stopPolling]);

  /** Call after triggering a manual sync to immediately refresh and start polling. */
  const handleTrigger = useCallback(async () => {
    await refresh();
    startPolling();
  }, [refresh, startPolling]);

  return {statuses, handleTrigger};
}
