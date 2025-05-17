import {MouseEvent, useState} from "react";
import {Checkbox, Grid, IconButton, Stack, Typography} from "@mui/material";
import FolderAvatar from "@/components/FolderAvatar";
import {CheckOutlined, Delete} from "@mui/icons-material";
import styles from './MediaLibrary.module.scss';
import {Sortable, SortableItem} from "@/components/Sortable";
import sortableListStyles from "@/pages/MediaLibrary/SortableList.module.scss";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {useNavigate} from "react-router";
import {DragStartEvent} from "@dnd-kit/core";

interface Props {
  folders: LibraryEntry[];
  onSortEnd: (folders: LibraryEntry[]) => void;
  onDelete: (folder: LibraryEntry) => void;
  selectedItemIds: number[];
  onSelect: (e: React.MouseEvent, id: number) => void;
}

export default function FolderList({folders, onSortEnd, onDelete, selectedItemIds, onSelect}: Props) {
  const navigate = useNavigate();

  const handleDelete = (event: MouseEvent, folder: LibraryEntry) => {
    event.preventDefault();
    event.stopPropagation();
    onDelete(folder);
  }

  const handleClick = (event: MouseEvent, id?: number) => {
    event.preventDefault();
    event.stopPropagation();
    navigate(`/mediaLibrary/${id}`);
  }

  return (
    <Grid container gap={1}>
      <Sortable<LibraryEntry>
        items={folders}
        onDragEnd={onSortEnd}
        selectedItemIds={selectedItemIds}
        getItemId={item => item.id}
      >
        {(visibleItems) => (
          <>
            {visibleItems.map(folder => (
              <SortableItem itemId={folder.id!} key={folder.id}>
                {(props, isDragging) => {
                  return (
                    <div
                      {...props}
                      key={folder.id}
                      className={[
                        styles.entry,
                        sortableListStyles.sortableListItem,
                        isDragging ? sortableListStyles.isDragging : ''
                      ].join(' ')}
                      onClick={(e) => handleClick(e, folder.id)}
                    >
                      <div className={styles.checkbox}>
                        <Checkbox checked={selectedItemIds.includes(folder.id!)} onClick={e => onSelect(e, folder.id!)}/>
                      </div>
                      {!!folder.playedAt && (
                        <div className={styles.playedSign}>
                          <CheckOutlined sx={{fontSize: '32px', color: 'green'}}/>
                        </div>
                      )}
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
                  )
                }}
              </SortableItem>
            ))}
          </>
        )}
      </Sortable>
    </Grid>
  )
}
