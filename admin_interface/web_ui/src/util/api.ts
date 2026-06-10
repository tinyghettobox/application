import {SystemConfig} from "@db-models/SystemConfig";
import {SpotifyConfig} from "@db-models/SpotifyConfig";
import {LibraryEntry, NewLibraryEntry} from "@db-models/LibraryEntry";
import {CreateSyncConfig, SyncConfig, SyncStatus} from "@/types/sync";

function snakeToCamel(some: string): string {
  return some.replace(/([a-z])_([a-z])/g, (_, a, b) => `${a}${b.toUpperCase()}`);
}

function camelToSnake(some: string): string {
  return some.replace(/([a-z])([A-Z])/g, (_, a, b) => `${a}_${b.toLowerCase()}`);
}

function convertCaseDeep<T>(convertFn: (key: string) => string, some: T): T {
  if (Array.isArray(some)) {
    return some.map((entry) => convertCaseDeep(convertFn, entry)) as any;
  }
  if (some !== null && typeof some === 'object') {
    const camel: any = {};
    for (const key in some) {
      if (key in some) {
        const value = some[key];
        camel[convertFn(key)] = convertCaseDeep(convertFn, value);
      }
    }
    return camel;
  }

  return some;
}

async function api<T>(method: 'GET' | 'POST' | 'PUT' | 'DELETE', path: string, payload?: unknown): Promise<T> {
  const requestInit: RequestInit = {method};
  if (payload && method === 'POST' || method === 'PUT') {
    requestInit.headers = {'Content-Type': 'application/json'};
    requestInit.body = JSON.stringify(convertCaseDeep(camelToSnake, payload));
  }
  const response = await fetch(path, requestInit);

  if (!response.ok) {
    throw new Error('Failed to send ' + path + ': ' + await response.text());
  }
  if (response.headers.get('Content-Type') === 'application/json') {
    return convertCaseDeep(snakeToCamel, await response.json());
  }

  return await response.text() as T;
}

export function upload(path: string, form: FormData, onProgress: (progress: number) => void, onLoad: (error?: string, data?: unknown) => void): void {
  const xhr = new XMLHttpRequest();
  xhr.open('POST', path, true);
  xhr.upload.addEventListener('progress', (event: ProgressEvent) => {
    if (event.lengthComputable) {
      onProgress(event.loaded / event.total);
    }
  });
  xhr.upload.addEventListener('error', () => {
    onLoad(xhr.statusText);
  });
  xhr.addEventListener('load', () => {
    if (xhr.status >= 200 && xhr.status < 300) {
      try {
        onLoad(undefined, JSON.parse(xhr.responseText));
      } catch (e) {
        onLoad(`${e}`)
      }
    } else {
      onLoad(xhr.statusText);
    }
  });
  xhr.addEventListener('error', () => {
    onLoad('Network error');
  });
  xhr.send(form);
}

async function get<T>(path: string): Promise<T> {
  return api('GET', path);
}

async function put<T, R = T>(path: string, payload: T): Promise<R> {
  return api('PUT', path, payload);
}

async function post<T, R = T>(path: string, payload: T): Promise<R> {
  return api('POST', path, payload);
}

async function del<T>(path: string): Promise<T> {
  return api('DELETE', path);
}

export async function getSystemConfig(): Promise<SystemConfig> {
  return get<SystemConfig>('/api/system/config');
}

export async function putSystemConfig(config: SystemConfig): Promise<SystemConfig> {
  return put<SystemConfig>('/api/system/config', config);
}

export async function getSpotifyConfig(): Promise<SpotifyConfig> {
  return get<SpotifyConfig>('/api/spotify/config');
}

export async function putSpotifyConfig(config: SpotifyConfig): Promise<SpotifyConfig> {
  return put<SpotifyConfig>('/api/spotify/config', config);
}

export async function getLibraryEntry(id: number): Promise<LibraryEntry> {
  return get<LibraryEntry>(`/api/library/${id}`);
}

export async function putLibraryEntry(id: number, entry: LibraryEntry): Promise<LibraryEntry> {
  return put<LibraryEntry>(`/api/library/${id}`, entry);
}

export async function delLibraryEntry(id: number): Promise<void> {
  return del<void>(`/api/library/${id}`);
}

export type LibraryEntryBulkUpdate = Pick<LibraryEntry, 'id'> & Partial<Pick<LibraryEntry, 'sortKey' | 'playedAt'>>

export async function postBulkLibraryEntries<T = LibraryEntryBulkUpdate[]>(update: T): Promise<void> {
  return post<T, void>('/api/library/bulk-update', update);
}

export async function postMarkLibraryEntriesPlayed(libraryEntryIds: number[], playedAt: string | null): Promise<void> {
  return post<any, void>('/api/library/mark-played', {libraryEntryIds, playedAt});
}

export async function postLibraryEntries(parent_id: number, entries: NewLibraryEntry[]): Promise<LibraryEntry[]> {
  return post<NewLibraryEntry[], LibraryEntry[]>(`/api/library?parent_id=${parent_id}`, entries);
}

export async function getSyncConfig(id: number): Promise<SyncConfig | null> {
  try {
    return await get<SyncConfig>(`/api/library/${id}/sync-config`);
  } catch {
    return null;
  }
}

export async function putSyncConfig(id: number, config: CreateSyncConfig): Promise<SyncConfig> {
  return put<CreateSyncConfig, SyncConfig>(`/api/library/${id}/sync-config`, config);
}

export async function getSyncStatus(id: number): Promise<SyncStatus | null> {
  try {
    return await get<SyncStatus>(`/api/library/${id}/sync-status`);
  } catch {
    return null;
  }
}

export async function getChildrenSyncStatuses(parentId: number): Promise<SyncStatus[]> {
  try {
    return await get<SyncStatus[]>(`/api/library/${parentId}/children/sync-statuses`);
  } catch {
    return [];
  }
}

export async function postTriggerSync(id: number): Promise<void> {
  return post<void, void>(`/api/library/${id}/sync`, undefined);
}

export function uploadLibraryEntryFile(file: File, onProgress: (progress: number) => void, onLoad: (error?: string, data?: unknown) => void) {
  const formData = new FormData();
  formData.append('name', file.name);
  formData.append('track', file);
  upload('/api/library/upload', formData, onProgress, onLoad);
}

