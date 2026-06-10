export type SortBy =
  | 'release_date_desc'
  | 'release_date_asc'
  | 'alpha_asc'
  | 'alpha_desc'
  | 'track_number_asc'
  | 'episode_regex_asc'
  | 'episode_regex_desc'
  | 'most_recently_played'
  | 'manual';

export const SORT_BY_LABELS: Record<SortBy, string> = {
  release_date_desc: 'Newest release first',
  release_date_asc: 'Oldest release first',
  alpha_asc: 'A → Z',
  alpha_desc: 'Z → A',
  track_number_asc: 'Track number',
  episode_regex_asc: 'Episode number (ascending)',
  episode_regex_desc: 'Episode number (descending)',
  most_recently_played: 'Most recently played',
  manual: 'Manual (drag to sort)',
};

export const SORT_BY_OPTIONS: SortBy[] = Object.keys(SORT_BY_LABELS) as SortBy[];

export type SyncIntervalDays = 0 | 1 | 7 | 30;

export const SYNC_INTERVAL_LABELS: Record<SyncIntervalDays, string> = {
  0: 'Manual only',
  1: 'Daily',
  7: 'Weekly',
  30: 'Monthly (default)',
};

export type SyncConfig = {
  libraryEntryId: number;
  sortBy: SortBy;
  sortRegex?: string;
  syncIntervalDays: SyncIntervalDays;
  lastSyncedAt?: string;
};

export type CreateSyncConfig = {
  sortBy: SortBy;
  sortRegex?: string;
  syncIntervalDays: number;
};

export type SyncStatusKind = 'pending' | 'running' | 'done' | 'error';

export type SyncStatus = {
  libraryEntryId: number;
  status: SyncStatusKind;
  itemsDone: number;
  itemsTotal?: number;
  errorMessage?: string;
  startedAt?: string;
  syncedAt?: string;
};

/** Default sync config for a given Spotify type. */
export function defaultSyncConfig(spotifyType: string): CreateSyncConfig {
  if (spotifyType === 'album' || spotifyType === 'playlist') {
    return { sortBy: 'track_number_asc', syncIntervalDays: 30 };
  }
  if (spotifyType === 'show') {
    return { sortBy: 'release_date_desc', syncIntervalDays: 30 };
  }
  // artist
  return { sortBy: 'release_date_desc', syncIntervalDays: 30 };
}

/** Container types that support sync. */
export const SYNCABLE_TYPES = ['artist', 'album', 'playlist', 'show'];

export function isSyncableType(spotifyType?: string): boolean {
  return !!spotifyType && SYNCABLE_TYPES.includes(spotifyType);
}
