import {ChangeEvent, useRef, useState} from "react";
import {Box, FormControl, Grid, IconButton, InputLabel, OutlinedInput} from "@mui/material";
import {Check, Delete} from "@mui/icons-material";
import {useAddEntryState} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";
import {LibraryEntry} from "@db-models/LibraryEntry";

export default function StreamAddForm() {
  const {entries, setEntries, addEntry, removeEntry, getNextSortKey} = useAddEntryState();
  const [newStreamUrl, setNewStreamUrl] = useState('');
  const [newStreamName, setNewStreamName] = useState('');
  const lastChange = useRef<string>();

  const handleNewStreamUrlChange = (event: ChangeEvent<HTMLInputElement>) => {
    setNewStreamUrl(event.target.value);
    lastChange.current = 'url';
  }

  const handleNewStreamNameChange = (event: ChangeEvent<HTMLInputElement>) => {
    setNewStreamName(event.target.value);
    lastChange.current = 'name';
  }

  const handleAdd = () => {
    if (!newStreamName || !newStreamUrl) {
      return;
    }
    debugger;
    addEntry({variant: 'stream', name: newStreamName, trackSource: {title: newStreamName, url: newStreamUrl}, sortKey: getNextSortKey()});
    setNewStreamUrl('');
    setNewStreamName('');
  }


  const handleUrlChange = (event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>, entry: LibraryEntry) => {
    const index = entries.indexOf(entry);
    if (index === -1) {
      throw new Error(`Can not find index of ${JSON.stringify(entry)} in playlist items: ${JSON.stringify(entries)}`);
    }

    if (typeof entries[index]?.trackSource !== 'undefined') {
      entries[index].trackSource!.url = event.target.value;
    }

    setEntries([...entries]);
  }

  const handleNameChange = (event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>, entry: LibraryEntry) => {
    const index = entries.indexOf(entry);
    if (index === -1) {
      throw new Error(`Can not find index of ${JSON.stringify(entry)} in playlist items: ${JSON.stringify(entries)}`);
    }

    entries[index].name = event.target.value;

    setEntries([...entries]);
  }

  return (
    <Box sx={{mt: 2}}>
      {entries.map((entry, index) =>
        <Grid container spacing={2} key={index}>
          <Grid item xs={5}>
            <FormControl fullWidth sx={{mb: 3}} size="small">
              <InputLabel>{index + 1}. Stream Name</InputLabel>
              <OutlinedInput
                label={`${index + 1}. Stream Name`}
                onChange={(event) => handleNameChange(event, entry)}
                value={entry.name}
              />
            </FormControl>
          </Grid>
          <Grid item xs={6}>
            <FormControl fullWidth sx={{mb: 3}} size="small">
              <InputLabel>{index + 1}. Stream URL</InputLabel>
              <OutlinedInput
                label={`${index + 1}. Stream URL`}
                onChange={(event) => handleUrlChange(event, entry)}
                value={entry.trackSource?.url}
              />
            </FormControl>
          </Grid>
          <Grid item xs={1}>
            <IconButton onClick={() => removeEntry(entry)} edge="end">
              <Delete />
            </IconButton>
          </Grid>
        </Grid>
      )}
      <Grid container spacing={2}>
        <Grid item xs={5}>
          <FormControl fullWidth sx={{mb: 3}} size="small">
            <InputLabel>{entries.length + 1}. Stream Name</InputLabel>
            <OutlinedInput
              label={`${entries.length + 1}. Stream Name`}
              value={newStreamName}
              onChange={handleNewStreamNameChange}
            />
          </FormControl>
        </Grid>
        <Grid item xs={6}>
          <FormControl fullWidth sx={{mb: 3}} size="small">
            <InputLabel>{entries.length + 1}. Stream URL</InputLabel>
            <OutlinedInput
              label={`${entries.length + 1}. Stream URL`}
              value={newStreamUrl}
              onChange={handleNewStreamUrlChange}
            />
          </FormControl>
        </Grid>
        <Grid item xs={1}>
          <IconButton onClick={handleAdd} edge="end" disabled={!newStreamName || !newStreamUrl}>
            <Check />
          </IconButton>
        </Grid>
      </Grid>
    </Box>
  )
}