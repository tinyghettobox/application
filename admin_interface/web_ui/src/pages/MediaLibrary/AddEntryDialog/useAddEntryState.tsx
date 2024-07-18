import {createContext, ReactElement, useContext, useState, MouseEvent} from "react";
import {notify} from "@/components/Notification";
import isEqual from "fast-deep-equal";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {postLibraryEntries} from "@/util/api";

type AddEntryState = {
  parentId?: number;
  entries: LibraryEntry[];
  setEntries: (entries: LibraryEntry[] | ((oldEntries: LibraryEntry[]) => LibraryEntry[])) => void;
  addEntry: (entry: LibraryEntry) => void;
  removeEntry: (entry: LibraryEntry) => void;
  isEntryAdded: (entry: LibraryEntry) => boolean;
  abort: () => void;
  submit: (event: MouseEvent) => void;
  getNextSortKey: () => number;
}

const AddEntryStateContext = createContext<AddEntryState | undefined>(undefined);

interface Props {
  parent: LibraryEntry;
  onClose: (submitted?: true) => void;
  children: ReactElement[] | ReactElement;
}

type FlatEntry = { parent?: LibraryEntry, entry: LibraryEntry };

function flatten(entries: LibraryEntry[], parent?: LibraryEntry): FlatEntry[] {
  return entries.flatMap(entry => {
    if (entry.variant === 'folder') {
      return [{parent, entry}, ...flatten(entry.children || [], entry)];
    }

    return [{parent, entry}];
  });
}

export const AddEntryStateProvider = (props: Props) => {
  const [entries, setEntries] = useState<LibraryEntry[]>([]);

  const addEntry = (entry: LibraryEntry) => {
    setEntries(oldEntries => [...oldEntries, entry]);
  }

  const removeEntry = (entry: LibraryEntry) => {
    setEntries(existingEntries => {
      let existingFlatEntries = flatten(existingEntries);

      for (const existingFlatEntry of existingFlatEntries) {
        if (existingFlatEntry.entry.id === entry.id) {
          const parent = existingFlatEntry.parent;
          if (parent && parent.variant === 'folder') {
            parent.children = (parent.children || []).filter(child => child.id !== entry.id);
          } else {
            existingEntries = existingEntries.filter(existingEntry => existingEntry.id !== entry.id);
          }
          break;
        }
      }

      return [...existingEntries];
    });
  }

  const isEntryAdded = (entry: LibraryEntry) => {
    return flatten(entries).some(flatEntry => isEqual(flatEntry.entry, entry));
  }

  const abort = () => {
    setEntries([]);
    props.onClose();
  }

  const submit = async (event: MouseEvent) => {
    event.preventDefault();

    try {
      await postLibraryEntries(props.parent.id as number, entries);
    } catch (e) {
      notify('error', `Error while creating entries: ${e}`, 8000);
      return;
    }

    notify('success', 'Entries created', 2000);
    setEntries([]);
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
    getNextSortKey
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