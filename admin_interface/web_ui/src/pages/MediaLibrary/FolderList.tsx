import {MouseEvent} from "react";
import {Box, Checkbox, Grid, IconButton, Stack, Tooltip, Typography} from "@mui/material";
import FolderAvatar from "@/components/FolderAvatar";
import {CheckOutlined, Delete, BlockOutlined} from "@mui/icons-material";
import styles from './MediaLibrary.module.scss';
import {Sortable, SortableItem} from "@/components/Sortable";
import sortableListStyles from "@/pages/MediaLibrary/SortableList.module.scss";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {useNavigate} from "react-router";
import SyncStatusChip from "@/pages/MediaLibrary/SyncStatusChip";
import {SyncStatus} from "@/types/sync";

interface Props {
  folders: LibraryEntry[];
  onSortEnd: (folders: LibraryEntry[]) => void;
  onDelete: (folder: LibraryEntry) => void;
  selectedItemIds: number[];
  onSelect: (e: React.MouseEvent, id: number) => void;
  syncStatuses: Map<number, SyncStatus>;
  onTriggerSync: () => void;
}

export default function FolderList({folders, onSortEnd, onDelete, selectedItemIds, onSelect, syncStatuses, onTriggerSync}: Props) {
  const navigate = useNavigate();

  const handleDelete = (event: MouseEvent, folder: LibraryEntry) => {
    event.preventDefault();
    event.stopPropagation();
    onDelete(folder);
  }

  const handleClick = (event: MouseEvent, id?: number, isDeleted?: boolean) => {
    if (isDeleted) return;
    event.preventDefault();
    event.stopPropagation();
    navigate(`/mediaLibrary/${id}`);
  }

  return (
    <Grid container spacing={1}>
      <Sortable<LibraryEntry>
        items={folders}
        onDragEnd={onSortEnd}
        selectedItemIds={selectedItemIds}
        getItemId={item => item.id}
      >
        {(visibleItems) => (
          <>
            {visibleItems.map(folder => (
              <Grid item xs={6} sm={4} md={3} key={folder.id}>
                <SortableItem itemId={folder.id!}>
                  {(props, isDragging) => {
                    return (
                      <div
                        {...props}
                        className={[
                          styles.entry,
                          sortableListStyles.sortableListItem,
                          isDragging ? sortableListStyles.isDragging : '',
                          folder.deleted ? styles.deletedEntry : '',
                        ].join(' ')}
                        onClick={(e) => handleClick(e, folder.id, folder.deleted)}
                      >
                        <div className={styles.checkbox}>
                          <Checkbox checked={selectedItemIds.includes(folder.id!)} onClick={e => onSelect(e, folder.id!)}/>
                        </div>
                        <Stack sx={{textAlign: 'center', opacity: folder.deleted ? 0.4 : 1, alignItems: 'center'}}>
                          <Box sx={{position: 'relative', display: 'inline-block', width: '100%'}}>
                            <FolderAvatar sx={{width: '100%', height: 'auto', aspectRatio: '1'}} folder={folder}/>
                            {!!folder.playedAt && !folder.deleted && (
                              <CheckOutlined sx={{fontSize: '28px', color: 'green', position: 'absolute', bottom: 0, right: 0}}/>
                            )}
                            {folder.deleted && (
                              <Tooltip title="No longer available on Spotify">
                                <BlockOutlined sx={{fontSize: '28px', color: 'text.disabled', position: 'absolute', bottom: 0, right: 0}}/>
                              </Tooltip>
                            )}
                          </Box>
                          <Typography
                            variant="subtitle2"
                            sx={{
                              width: '100%',
                              textDecoration: folder.deleted ? 'line-through' : 'none',
                              color: folder.deleted ? 'text.disabled' : 'inherit',
                              wordBreak: 'break-word',
                            }}
                            className={styles.name}
                          >{folder.name}</Typography>
                          {folder.id && !folder.deleted && (
                            <SyncStatusChip status={syncStatuses.get(folder.id) ?? null} onRefresh={onTriggerSync}/>
                          )}
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
                      </div>
                    )
                  }}
                </SortableItem>
              </Grid>
            ))}
          </>
        )}
      </Sortable>
    </Grid>
  )
}
