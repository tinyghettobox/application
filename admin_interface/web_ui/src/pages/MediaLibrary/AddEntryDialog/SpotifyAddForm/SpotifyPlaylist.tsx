import {Avatar, IconButton, List, ListItem, ListItemAvatar, ListItemText, Typography} from "@mui/material";
import {Delete} from "@mui/icons-material";
import {useAddEntryState} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";
import {Sortable, SortableItem} from "@/components/Sortable";
import styles from "./SpotifyAddForm.module.scss";
import sortableListStyles from "../../SortableList.module.scss";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {arrayToBase64} from "@/util/base64";

export default function SpotifyPlaylist() {
  const {entries, setEntries, removeEntry} = useAddEntryState();

  const handleDragEnd = (itemIds: string[]) => {
    setEntries(entries =>
      itemIds.map(id => entries.find(entry => getId(entry) === id) as LibraryEntry)
    )
  }

  const handleDelete = (entry: LibraryEntry) => {
    removeEntry(entry);
  }

  const getId = (entry: LibraryEntry) => {
    return entry.trackSource?.spotifyId as string;
  }

  return (
    <div>
      <Typography variant="h5" className={styles.title}>Playlist</Typography>
      {entries.length === 0 && <Typography variant="body2" sx={{mt: 2}}>No items added yet</Typography>}

      <Sortable itemIds={entries.map(getId)} onDragEnd={handleDragEnd}>
        <List>
          {entries.map((entry, index) => (
            <SortableItem itemId={getId(entry)} key={getId(entry)}>
              {(props, isDragging) => (
                <ListItem
                  {...props}
                  className={[
                    sortableListStyles.sortableListItem,
                    isDragging ? sortableListStyles.isDragging : ''
                  ].join(' ')}
                  secondaryAction={<IconButton onClickCapture={() => handleDelete(entry)}><Delete/></IconButton>}
                >
                  <ListItemAvatar>
                    <Avatar src={`data:image/png;base64,${arrayToBase64(entry.image || [])}`} alt={`Avatar for ${entry.name}`}/>
                  </ListItemAvatar>
                  <ListItemText primary={<Typography variant="body2">{entry.name}</Typography>}/>
                </ListItem>
              )}
            </SortableItem>
          ))}
        </List>
      </Sortable>
    </div>
  )
}