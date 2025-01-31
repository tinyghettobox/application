import {ChangeEvent, useState} from "react";
import {Box, Button, CircularProgress, IconButton, List, ListItem, ListItemAvatar, ListItemText, styled} from "@mui/material";
import {Check, Delete, ErrorOutline, Upload} from "@mui/icons-material";
import {useAddEntryState} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";
import {Sortable, SortableItem} from "@/components/Sortable";
import sortableListStyles from "../../SortableList.module.scss";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {uploadLibraryEntryFile} from "@/util/api";
import {notify} from "@/components/Notification";


const VisuallyHiddenInput = styled('input')({
  clip: 'rect(0 0 0 0)',
  clipPath: 'inset(50%)',
  height: 1,
  overflow: 'hidden',
  position: 'absolute',
  bottom: 0,
  left: 0,
  whiteSpace: 'nowrap',
  width: 1,
});

export default function FileAddForm() {
  const {entries, setEntries, removeEntry, getNextSortKey} = useAddEntryState();
  const [uploadProgress, setUploadProgress] = useState<{ [name: string]: number }>({});

  const handleUpload = async (event: ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files?.length) {
      return;
    }

    let sortKey = getNextSortKey();
    const newEntries: LibraryEntry[] = [];
    for (let i = 0; i < event.target.files.length; i++) {
      const file = event.target.files?.[i] as File;

      newEntries.push({name: file.name, variant: 'file', trackSource: {title: file.name || ''}, sortKey: sortKey++});

      uploadLibraryEntryFile(
        file,
        (progress) => {
          setUploadProgress(prev => ({...prev, [file.name]: progress}));
        },
        (error) => {
          if (error) {
            notify('error', `Failed to upload ${file.name}: ${error}`, 6000);
          }
          setUploadProgress(prev => ({...prev, [file.name]: !error ? 100 : -1}));
        }
      );
    }

    event.target.value = '';

    setEntries((oldEntries) => [
      ...oldEntries,
      ...newEntries
    ]);
  }

  const handleDragEnd = (itemIds: string[]) => {
    setEntries(entries =>
      itemIds.map(id => entries.find(entry => entry.name === id) as LibraryEntry)
    );
  };

  const handleDelete = (entry: LibraryEntry) => {
    removeEntry(entry);
  }

  return (
    <Box sx={{mt: 2}}>
      <Box sx={{textAlign: 'center', mb: 1}}>
        <Button component="label" variant="outlined" startIcon={<Upload/>}>
          Upload tracks
          <VisuallyHiddenInput type="file" name="image" accept="audio/*" multiple onChange={handleUpload}/>
        </Button>
      </Box>
      <Sortable itemIds={(entries as LibraryEntry[]).map(entry => entry.name)} onDragEnd={handleDragEnd}>
        <List>
          {(entries as LibraryEntry[]).map((entry) =>
            <SortableItem itemId={entry.name} key={entry.name}>
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
                    {uploadProgress[entry.name] == 100 ? (
                      <Check/>
                    ) : uploadProgress[entry.name] == -1 ? (
                      <ErrorOutline color={"error"}/>
                    ) : (
                      <CircularProgress value={uploadProgress[entry.name] || 0}/>
                    )}
                  </ListItemAvatar>
                  <ListItemText
                    primary={entry.name}
                    secondary={`Filename: ${entry.trackSource?.title || entry.name}`}
                    sx={{wordWrap: 'break-word'}}
                  />
                </ListItem>
              )}
            </SortableItem>
          )}
        </List>
      </Sortable>
    </Box>
  );
}