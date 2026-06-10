import {createContext, ReactElement, useContext, useState, MouseEvent} from "react";
import {notify} from "@/components/Notification";
import isEqual from "fast-deep-equal";
import {LibraryEntry, NewLibraryEntry} from "@db-models/LibraryEntry";
import {postLibraryEntries, putSyncConfig, postTriggerSync} from "@/util/api";
import {CreateSyncConfig, defaultSyncConfig, isSyncableType} from "@/types/sync";

type AddEntryState = {
  parentId?: number;
  entries: NewLibraryEntry[];
  setEntries: (entries: NewLibraryEntry[] | ((oldEntries: NewLibraryEntry[]) => NewLibraryEntry[])) => void;
  addEntry: (entry: NewLibraryEntry) => void;
  removeEntry: (entry: NewLibraryEntry) => void;
  isEntryAdded: (entry: NewLibraryEntry) => boolean;
  abort: () => void;
  submit: (event: MouseEvent) => void;
  getNextSortKey: () => number;
  syncConfigs: Map<string, CreateSyncConfig>;
  setSyncConfigForEntry: (spotifyId: string, config: CreateSyncConfig) => void;
}

const AddEntryStateContext = createContext<AddEntryState | undefined>(undefined);

interface Props {
  parent: LibraryEntry;
  onClose: (submitted?: true) => void;
  children: ReactElement[] | ReactElement;
}

type FlatEntry = { parent?: NewLibraryEntry, entry: NewLibraryEntry };

function flatten(entries: NewLibraryEntry[], parent?: NewLibraryEntry): FlatEntry[] {
  return entries.flatMap(entry => {
    if (entry.variant === 'folder') {
      return [{parent, entry}, ...flatten(entry.children || [], entry)];
    }

    return [{parent, entry}];
  });
}

export const AddEntryStateProvider = (props: Props) => {
  const [entries, setEntries] = useState<NewLibraryEntry[]>([]);
  const [syncConfigs, setSyncConfigs] = useState<Map<string, CreateSyncConfig>>(new Map());

  const addEntry = (entry: NewLibraryEntry) => {
    setEntries(oldEntries => [...oldEntries, entry]);
  }

  const removeEntry = (entry: NewLibraryEntry) => {
    setEntries(existingEntries => {
      let existingFlatEntries = flatten(existingEntries);

      for (const existingFlatEntry of existingFlatEntries) {
        if (existingFlatEntry.entry === entry) {
          const parent = existingFlatEntry.parent;
          if (parent && parent.variant === 'folder') {
            parent.children = (parent.children || []).filter(child => child !== entry);
          } else {
            existingEntries = existingEntries.filter(existingEntry => existingEntry !== entry);
          }
          break;
        }
      }

      return [...existingEntries];
    });
  }

  const isEntryAdded = (entry: NewLibraryEntry) => {
    return flatten(entries).some(flatEntry => isEqual(flatEntry.entry, entry));
  }

  const abort = () => {
    setEntries([]);
    setSyncConfigs(new Map());
    props.onClose();
  }

  const setSyncConfigForEntry = (spotifyId: string, config: CreateSyncConfig) => {
    setSyncConfigs(prev => new Map(prev).set(spotifyId, config));
  };

  const submit = async (event: MouseEvent) => {
    event.preventDefault();

    let createdEntries: LibraryEntry[];
    try {
      createdEntries = await postLibraryEntries(props.parent.id as number, entries);
    } catch (e) {
      notify('error', `Error while creating entries: ${e}`, 8000);
      return;
    }

    // Post sync configs for Spotify container entries and trigger immediate sync.
    for (const created of createdEntries) {
      const spotifyId = created.trackSource?.spotifyId;
      const spotifyType = created.trackSource?.spotifyType;
      if (!spotifyId || !isSyncableType(spotifyType)) continue;

      const config = syncConfigs.get(spotifyId) ?? defaultSyncConfig(spotifyType!);

      try {
        await putSyncConfig(created.id!, config);
        // putSyncConfig already sets status=pending which starts the job,
        // but we call trigger explicitly to make it visible immediately.
        await postTriggerSync(created.id!);
      } catch (e) {
        notify('error', `Sync config error for ${created.name}: ${e}`, 6000);
      }
    }

    notify('success', 'Entries created', 2000);
    setEntries([]);
    setSyncConfigs(new Map());
    props.onClose(true);
  }

  const getNextSortKey = () => {
    if (entries.length) {
      return 1 + Math.max(...entries.map(child => child.sortKey));
    }
    if (props.parent.children?.length) {
      return 1 + Math.max(...props.parent.children.map(child => child.sortKey));
    }
    return 0;
  }

  const contextValue = {
    parentId: props.parent.id,
    entries,
    setEntries,
    addEntry,
    removeEntry,
    isEntryAdded,
    abort,
    submit,
    getNextSortKey,
    syncConfigs,
    setSyncConfigForEntry,
  }


  return <AddEntryStateContext.Provider value={contextValue}>{props.children}</AddEntryStateContext.Provider>
}

export function useAddEntryState(): AddEntryState {
  const context = useContext(AddEntryStateContext);
  if (!context) {
    throw new Error('useAddTrackState must be used within a AddEntryStateProvider');
  }

  return context as unknown as AddEntryState;
}