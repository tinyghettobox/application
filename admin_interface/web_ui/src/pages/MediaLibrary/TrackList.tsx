import {IconButton, List, ListItem, ListItemText} from "@mui/material";
import {Sortable, SortableItem} from "@/components/Sortable";
import sortableListStyles from "./SortableList.module.scss";
import {Delete} from "@mui/icons-material";
import {LibraryEntry} from "@db-models/LibraryEntry";

interface Props {
  tracks: LibraryEntry[];
  onSortEnd: (itemIds: string[]) => void;
  onDelete: (track: LibraryEntry) => void;
}

export default function TrackList({tracks, onSortEnd, onDelete}: Props) {
  return (
    <div>
      <Sortable itemIds={tracks.map(track => `${track.id}`)} onDragEnd={onSortEnd}>
        <List>
          {tracks.map(track => (
            <SortableItem itemId={`${track.id}`} key={track.id}>
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
      </Sortable>
    </div>
  )
}