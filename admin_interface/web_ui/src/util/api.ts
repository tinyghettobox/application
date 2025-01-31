import {SystemConfig} from "@db-models/SystemConfig";
import {SpotifyConfig} from "@db-models/SpotifyConfig";
import {LibraryEntry} from "@db-models/LibraryEntry";

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

export function upload(path: string, form: FormData, onProgress: (progress: number) => void, onLoad: (error?: string) => void): void {
  const xhr = new XMLHttpRequest();
  xhr.open('POST', path, true);
  xhr.upload.addEventListener('progress', (event: ProgressEvent) => {
    if (event.lengthComputable) {
      onProgress(event.loaded / event.total);
    }
  });
  xhr.upload.addEventListener('error', () => {
    debugger;
    onLoad(xhr.statusText);
  });
  xhr.addEventListener('load', () => {
    if (xhr.status >= 200 && xhr.status < 300) {
      onLoad();
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

async function put<T>(path: string, payload: T): Promise<T> {
  return api('PUT', path, payload);
}

async function post<T>(path: string, payload: T): Promise<T> {
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

export async function postLibraryEntries(parent_id: number, entries: LibraryEntry[]): Promise<LibraryEntry[]> {
  return post<LibraryEntry[]>(`/api/library?parent_id=${parent_id}`, entries);
}

export function uploadLibraryEntryFile(file: File, onProgress: (progress: number) => void, onLoad: (error?: string) => void) {
  const formData = new FormData();
  formData.append('name', file.name);
  formData.append('track', file);
  upload('/api/library/upload', formData, onProgress, onLoad);
}

