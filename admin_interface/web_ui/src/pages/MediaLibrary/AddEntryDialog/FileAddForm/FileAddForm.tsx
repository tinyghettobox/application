import {ChangeEvent} from "react";
import {
  Box,
  Button,
  IconButton,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  styled
} from "@mui/material";
import {Check, Delete, Upload} from "@mui/icons-material";
import {useAddEntryState} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";
import {Sortable, SortableItem} from "@/components/Sortable";
import sortableListStyles from "../../SortableList.module.scss";
import {fileToBinary} from "@/pages/MediaLibrary/AddEntryDialog/helper";
import {LibraryEntry} from "@db-models/LibraryEntry";


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

  const handleUpload = async (event: ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files?.length) {
      return;
    }

    let sortKey = getNextSortKey();
    const newEntries: LibraryEntry[] = [];
    for (let i = 0; i < event.target.files.length; i++) {
      const file = event.target.files?.[i] as File;
      const binary = await fileToBinary(file);

      newEntries.push({name: file.name, variant: 'file', trackSource: { title: file.name || '', file: binary }, sortKey: sortKey++});
    }

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
                      <Check/>
                  </ListItemAvatar>
                  <ListItemText
                    primary={entry.name}
                    secondary={`Filename: ${entry.trackSource?.title || entry.name}`}
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