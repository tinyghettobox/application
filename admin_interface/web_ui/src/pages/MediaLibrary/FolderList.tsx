import {MouseEvent} from "react";
import {Grid, IconButton, Stack, Typography} from "@mui/material";
import FolderAvatar from "@/components/FolderAvatar";
import {Delete} from "@mui/icons-material";
import styles from './MediaLibrary.module.scss';
import {Sortable, SortableItem} from "@/components/Sortable";
import sortableListStyles from "@/pages/MediaLibrary/SortableList.module.scss";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {useNavigate} from "react-router";

interface Props {
  folders: LibraryEntry[]
  onSortEnd: (itemIds: string[]) => void;
  onDelete: (folder: LibraryEntry) => void;
}

export default function FolderList({folders, onSortEnd, onDelete}: Props) {
  const navigate = useNavigate();

  const handleDelete = (event: MouseEvent, folder: LibraryEntry) => {
    event.preventDefault();
    event.stopPropagation();
    onDelete(folder);
  }

  const onSelect = (event: MouseEvent, id?: number) => {
    event.preventDefault();
    event.stopPropagation();
    navigate(`/mediaLibrary/${id}`);
  }

  return (
    <Grid container gap={1}>
      <Sortable itemIds={folders.map(folder => `${folder.id}`)} onDragEnd={onSortEnd}>
        <>
          {folders.map(folder => (
            <SortableItem itemId={`${folder.id}`} key={folder.id}>
              {(props, isDragging) => (
                <div
                  {...props}
                  key={folder.id}
                  className={[
                    styles.entry,
                    sortableListStyles.sortableListItem,
                    isDragging ? sortableListStyles.isDragging : ''
                  ].join(' ')}
                  onClick={(e) => onSelect(e, folder.id)}
                >
                  <Grid item xs={'auto'} key={folder.id}>
                    <Stack sx={{textAlign: 'center'}}>
                      <FolderAvatar sx={{width: '180px', height: '180px'}} folder={folder}/>
                      <Typography
                        variant="subtitle1"
                        sx={{maxWidth: '180px'}}
                        className={styles.name}
                      >{folder.name}</Typography>
                      <div>
                        <IconButton
                          size="small"
                          color="error"
                          onClickCapture={(event) => handleDelete(event, folder)}
                          className={styles.deleteButton}
                        >
                          <Delete/>
                        </IconButton>
                      </div>
                    </Stack>
                  </Grid>
                </div>
              )}
            </SortableItem>
          ))}
        </>
      </Sortable>
    </Grid>
  )
}