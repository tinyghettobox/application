import {useCallback, useEffect, useState} from "react";
import {notify} from "@/components/Notification";
import {delLibraryEntry, getLibraryEntry, putLibraryEntry} from "@/util/api";
import {LibraryEntry} from "@db-models/LibraryEntry";

export function useLibraryEntry(id?: number) {
  const [state, setState] = useState<{
    libraryEntry?: LibraryEntry,
    loading: boolean,
    error?: string
  }>({loading: true});

  const loadLibraryEntry = useCallback(async () => {
    setState(state => ({...state, loading: true}));
    try {
      setState({loading: false, libraryEntry: await getLibraryEntry(id || 0)});
    } catch (e) {
      notify('error', `Could not load library entry: ${e}`);
      setState({
        loading: false,
        libraryEntry: undefined,
        error: `Could not load library entry: ${e}`
      });
    }
  }, [setState, id]);

  const deleteLibraryEntry = useCallback(async (id: number) => {
    try {
      await delLibraryEntry(id);
      setState(oldState => {
        if (oldState.libraryEntry) {
          oldState.libraryEntry.children = oldState.libraryEntry.children?.filter(child => child.id !== id);
        }
        return { ...oldState }
      });
      notify('success', `Entry deleted`, 2000);
      await loadLibraryEntry();
    } catch (e) {
      notify('error', `Could not delete library entry: ${e}`);
    }
  }, [loadLibraryEntry]);

  const updateLibraryEntry = useCallback(async (updateCb: (oldEntry: LibraryEntry) => LibraryEntry) => {
    if (!state.libraryEntry || !id) {
      throw new Error('No library entry to update');
    }
    const updated = updateCb(state.libraryEntry);
    setState(state => ({...state, loading: true, libraryEntry: updated}));

    try {
      let updatedEntry = await putLibraryEntry(id, updated);
      setState({loading: false, libraryEntry: updatedEntry});
      notify('success', `Saved`, 2000);
    } catch (e) {
      setState({
        loading: false,
        libraryEntry: undefined,
        error: `Could not update library entry: ${e}`
      });
      notify('error', `Could not load library entry: ${e}`);
    }
  }, [id, setState, state.libraryEntry]);

  useEffect(() => {
    loadLibraryEntry();
  }, [id, loadLibraryEntry]);

  return {...state, reloadLibraryEntry: loadLibraryEntry, deleteLibraryEntry, updateLibraryEntry};
}
