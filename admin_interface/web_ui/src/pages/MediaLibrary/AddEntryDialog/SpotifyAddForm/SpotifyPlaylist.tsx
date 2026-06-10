import {Avatar, Box, IconButton, List, ListItem, ListItemAvatar, ListItemText, Typography} from "@mui/material";
import {Delete} from "@mui/icons-material";
import {useAddEntryState} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";
import SyncConfigSection from "./SyncConfigSection";
import {CreateSyncConfig, defaultSyncConfig, isSyncableType} from "@/types/sync";
import {Sortable, SortableItem} from "@/components/Sortable";
import styles from "./SpotifyAddForm.module.scss";
import sortableListStyles from "../../SortableList.module.scss";
import {NewLibraryEntry} from "@db-models/LibraryEntry";
import {arrayToBase64} from "@/util/base64";

export default function SpotifyPlaylist() {
  const {entries, setEntries, removeEntry, syncConfigs, setSyncConfigForEntry} = useAddEntryState();

  const handleDragEnd = (items: NewLibraryEntry[]) => {
    setEntries(entries =>
      items.map(item => entries.find(entry => getId(entry) === getId(item)) as NewLibraryEntry)
    )
  }

  const handleDelete = (entry: NewLibraryEntry) => {
    removeEntry(entry);
  }

  const getId = (entry: NewLibraryEntry) => {
    return entry.trackSource?.spotifyId as string;
  }

  return (
    <div>
      <Typography variant="h5" className={styles.title}>Playlist</Typography>
      {entries.length === 0 && <Typography variant="body2" sx={{mt: 2}}>No items added yet</Typography>}

      <Sortable<NewLibraryEntry> items={entries} onDragEnd={handleDragEnd} getItemId={getId}>
        {(visibleItems) => (
          <List component={'div'}>
            {entries.map((entry) => {
              const spotifyId = entry.trackSource?.spotifyId;
              const syncable = isSyncableType(entry.trackSource?.spotifyType);
              const config: CreateSyncConfig | undefined = syncable && spotifyId
                ? (syncConfigs.get(spotifyId) ?? defaultSyncConfig(entry.trackSource!.spotifyType!))
                : undefined;

              return (
                <SortableItem itemId={getId(entry)} key={getId(entry)}>
                  {(props, isDragging) => (
                    <Box
                      {...props}
                      className={[
                        sortableListStyles.sortableListItem,
                        isDragging ? sortableListStyles.isDragging : ''
                      ].join(' ')}
                      sx={{mb: syncable ? 2 : 0, pl: 1, pr: 1}}
                    >
                      <ListItem
                        disableGutters
                        component="div"
                        secondaryAction={<IconButton onClickCapture={() => handleDelete(entry)}><Delete/></IconButton>}
                      >
                        <ListItemAvatar>
                          <Avatar src={`data:image/png;base64,${arrayToBase64(entry.image || [])}`} alt={`Avatar for ${entry.name}`}/>
                        </ListItemAvatar>
                        <ListItemText primary={<Typography variant="body2">{entry.name}</Typography>}/>
                      </ListItem>
                      {syncable && config && spotifyId && (
                        <Box sx={{pb: 1}}>
                          <SyncConfigSection
                            value={config}
                            onChange={(c) => setSyncConfigForEntry(spotifyId, c)}
                          />
                        </Box>
                      )}
                    </Box>
                  )}
                </SortableItem>
              );
            })}
          </List>
        )}
      </Sortable>
    </div>
  )
}