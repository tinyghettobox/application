import {Checkbox, IconButton, List, ListItem, ListItemIcon, ListItemText} from "@mui/material";
import {Sortable, SortableItem} from "@/components/Sortable";
import sortableListStyles from "./SortableList.module.scss";
import {Delete} from "@mui/icons-material";
import {LibraryEntry} from "@db-models/LibraryEntry";

interface Props {
  tracks: LibraryEntry[];
  onSortEnd: (tracks: LibraryEntry[]) => void;
  onDelete: (track: LibraryEntry) => void;
  selectedItemIds: number[];
  onSelect: (e: React.MouseEvent, id: number) => void;
}

export default function TrackList({tracks, onSortEnd, onDelete, selectedItemIds, onSelect}: Props) {
  return (
    <div>
      <Sortable<LibraryEntry>
        items={tracks}
        onDragEnd={onSortEnd}
        selectedItemIds={selectedItemIds}
        getItemId={item => item.id}
      >
        {(visibleItems) => (
          <List>
            {visibleItems.map(track => (
              <SortableItem itemId={track.id!} key={track.id}>
                {(props, isDragging) => (
                  <ListItem
                    {...props}
                    key={track.id}
                    className={[
                      sortableListStyles.sortableListItem,
                      isDragging ? sortableListStyles.isDragging : ''
                    ].join(' ')}
                    secondaryAction={<IconButton onClickCapture={() => onDelete(track)}><Delete/></IconButton>}
                  >
                    <ListItemIcon>
                      <Checkbox checked={selectedItemIds.includes(track.id!)} onClick={e => onSelect(e, track.id!)}/>
                    </ListItemIcon>
                    <ListItemText
                      primary={track.name}
                      secondary={
                        <>
                          {track.variant === 'file' && `Filename: ${track.trackSource?.title}`}
                          {track.variant === 'stream' && `URL: ${track.trackSource?.url}`}
                          {track.variant === 'spotify' && `SpotifyID: ${track.trackSource?.spotifyId}`}
                        </>
                      }
                    />
                  </ListItem>
                )}
              </SortableItem>
            ))}
          </List>
        )}
      </Sortable>
    </div>
  )
}
