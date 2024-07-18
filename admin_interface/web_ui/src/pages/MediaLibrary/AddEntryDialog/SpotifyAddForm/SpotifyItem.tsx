import {Avatar, Box, CircularProgress, IconButton, LinearProgress, Tooltip, Typography} from "@mui/material";
import {Add, Check, ChevronRight} from "@mui/icons-material";
import {MouseEvent, useState} from "react";
import {notify} from "@/components/Notification";
import styles from "./SpotifyAddForm.module.scss";
import {Variant} from "@db-models/Variant";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {useAddEntryState} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";
import {searchResultToLibraryEntry} from "@/pages/MediaLibrary/AddEntryDialog/SpotifyAddForm/helper";
import {ItemTypes} from "@fostertheweb/spotify-web-sdk";
import {arrayToBase64} from "@/util/base64";

interface Props {
  parentSelected?: boolean,
  entry: LibraryEntry,
  allowedVariant?: Variant
}

function getChildType(parentType: string): ItemTypes {
  const childType = {
    'artist': 'album',
    'album': 'track',
    'playlist': 'track',
    'show': 'episode'
  }[parentType] as ItemTypes;

  if (!childType) {
    throw new Error(`No child type known for ${parentType}`);
  }

  return childType;
}

async function loadChildren(itemType: string, id: string, offset = 0) {
  const params = new URLSearchParams({ parent_type: itemType, parent_id: id, offset: `${offset}` });
  const response = await fetch(`/api/spotify/children?${params}`);
  if (response.status !== 200) {
    notify('error', `Failed to fetch: ${await response.text()}`);
    return {limit: 0, total: 0};
  }
  return await response.json();
}

async function loadChildrenNested(item: LibraryEntry, onProgress?: (progress: number) => void) {
  if (!item.trackSource || !item.trackSource?.spotifyId || !item.trackSource?.spotifyType) {
    return [];
  }

  let children: LibraryEntry[] = [];
  let loadMode = true;
  let offset = 0;

  while (loadMode) {
    const result: any = await loadChildren(item.trackSource.spotifyType, item.trackSource.spotifyId, offset);

    if (Array.isArray(result.items)) {
      const childType = getChildType(item.trackSource.spotifyType);
      let sortKey = 0;
      for (const resultItem of result.items) {
        let entry: LibraryEntry;
        if (item.trackSource.spotifyType === 'playlist' && resultItem.track) {
          entry = await searchResultToLibraryEntry(resultItem.track, childType, sortKey++);
        } else {
          entry = await searchResultToLibraryEntry(resultItem, childType, sortKey++);
        }

        if (entry.variant === 'folder') {
          entry.children = await loadChildrenNested(entry);
        }

        children.push(entry);
        offset += 1;

        onProgress?.(100 / result.total * offset);
      }
    }

    if (result.limit + offset >= result.total) {
      loadMode = false;
    }
  }

  return children;
}

export default function SpotifyItem({entry, parentSelected = false, allowedVariant}: Props) {
  const {addEntry, removeEntry, isEntryAdded} = useAddEntryState();
  const [state, setState] = useState({
    loading: false,
    loadingProgress: 0,
    loadingAdd: false,
    libraryEntry: entry,
    childrenLoaded: !!entry.children,
    showing: false
  });

  const libraryEntryIsAdded = isEntryAdded(state.libraryEntry);

  const handleToggleExpand = async () => {
    if (state.loading || state.libraryEntry.variant !== 'folder') {
      return true;
    }
    if (state.childrenLoaded) {
      setState(state => ({...state, showing: !state.showing}));
      return;
    }

    setState(prevState => ({...prevState, loading: true, showing: true}));
    const children = await loadChildrenNested(state.libraryEntry, (progress) => {
      setState(prevState => ({...prevState, loadingProgress: progress}));
    });
    setState(prevState => ({
      ...prevState,
      loading: false,
      libraryEntry: {...prevState.libraryEntry, children},
      childrenLoaded: true
    }));
  }

  const handleAddRemoveEntry = async (event: MouseEvent) => {
    event.stopPropagation();
    // Always remove it and its children to ensure children are not individually added
    removeEntry(state.libraryEntry);

    if (!libraryEntryIsAdded) {
      addEntry(state.libraryEntry);
    }
  }
  const dataUri = state.libraryEntry.image ? `data:image/png;base64,${arrayToBase64(state.libraryEntry.image)}` : '';

  return (
    <div className={[styles.itemContainer, state.showing ? styles.expanded : ''].join(' ')}>
      <div className={styles.item} onClick={handleToggleExpand}>
        <div className={styles.avatar}>
          <Avatar alt={state.libraryEntry.name} src={dataUri}/>
        </div>
        <div className={styles.name}>
          {state.libraryEntry.variant === 'folder' && <ChevronRight className={styles.icon}/>}

          <Typography variant="body2">
            {state.libraryEntry.name}
          </Typography>
        </div>
        <div className={styles.action}>
          <Tooltip
            title={parentSelected ?
              'Parent is selected already' :
              allowedVariant && allowedVariant !== state.libraryEntry.variant ?
                `You can not add ${state.libraryEntry.variant}s in this folder as types of library entries in one folder need to be the same` :
                libraryEntryIsAdded ?
                  'Remove this item from the playlist' :
                  state.childrenLoaded ?
                    'Add this item to be played' :
                    'Please first toggle the item to load its children'}
          >
            <div>
              <IconButton
                onClick={handleAddRemoveEntry}
                disabled={parentSelected || !state.childrenLoaded || (allowedVariant && allowedVariant !== state.libraryEntry.variant)}
              >
                {libraryEntryIsAdded || parentSelected ?
                  <Check color={parentSelected ? 'disabled' : 'primary'}/> :
                  state.loadingAdd ?
                    <CircularProgress size={24}/> :
                    <Add/>
                }
              </IconButton>
            </div>
          </Tooltip>
        </div>
      </div>
      <div className={styles.itemChildren}>
        {state.libraryEntry.variant === 'folder' && state.libraryEntry.children?.map((child) =>
          <SpotifyItem
            key={child.id}
            entry={child}
            parentSelected={libraryEntryIsAdded || parentSelected}
            allowedVariant={allowedVariant}
          />
        )}
        {state.loading &&
          <Box sx={{mb: 2, mt: 1}}>
            <Typography variant={'subtitle2'}>Loading children</Typography>
            <LinearProgress variant="determinate" value={state.loadingProgress}/>
          </Box>
        }
      </div>
    </div>
  )
}