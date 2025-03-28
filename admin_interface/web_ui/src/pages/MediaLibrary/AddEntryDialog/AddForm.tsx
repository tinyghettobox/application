import {Button, FormControl, FormControlLabel, FormLabel, Grid, Radio, RadioGroup, Typography} from "@mui/material";
import SpotifyAddForm from "@/pages/MediaLibrary/AddEntryDialog/SpotifyAddForm/SpotifyAddForm";
import StreamAddForm from "@/pages/MediaLibrary/AddEntryDialog/StreamAddForm/StreamAddForm";
import FileAddForm from "@/pages/MediaLibrary/AddEntryDialog/FileAddForm/FileAddForm";
import {useState} from "react";
import FolderAddForm from "./FolderAddForm/FolderAddForm";
import {useAddEntryState} from "./useAddEntryState";
import {Variant} from "@db-models/Variant";

interface Props {
  allowedVariant?: Variant;
}

export default function AddForm({allowedVariant}: Props) {
  const {setEntries, abort, submit, entries} = useAddEntryState();
  const [sourceType, setSourceType] = useState<'folder' | 'file' | 'stream' | 'spotify' | 'all'>(allowedVariant === 'folder' ?
    'folder' :
    'spotify'
  );

  const handleSourceTypeChange = (_event: any, value: string) => {
    setSourceType(value as 'folder' | 'file' | 'stream' | 'spotify');
    setEntries([]);
  }

  return (
    <>
      <Grid container>
        <Grid item xs={6}>
          <Typography variant="h5">Add Entry</Typography>
        </Grid>
        <Grid item xs={6} sx={{textAlign: 'right'}}>
          <Button sx={{mr: 2}} onClick={abort}>Cancel</Button>
          <Button variant="contained" onClick={submit} disabled={entries.length === 0}>Submit</Button>
        </Grid>
      </Grid>
      <FormControl fullWidth>
        <FormLabel id="source-type-label">Type</FormLabel>
        <RadioGroup
          aria-labelledby="source-type-label"
          name={'sourceType'}
          value={sourceType}
          onChange={handleSourceTypeChange}
          sx={{pb: 1}}
          row
        >
          <FormControlLabel control={<Radio/>} label={'Folder'} value={'folder'} disabled={allowedVariant && allowedVariant !== 'folder'}/>
          <FormControlLabel
            control={<Radio/>}
            label={'Spotify'}
            value={'spotify'}
          />
          <FormControlLabel control={<Radio/>} label={'File'} value={'file'} disabled={allowedVariant === 'folder'}/>
          <FormControlLabel control={<Radio/>} label={'Stream'} value={'stream'} disabled={allowedVariant === 'folder'}/>
        </RadioGroup>
      </FormControl>

      {sourceType === 'folder' && <FolderAddForm/>}
      {sourceType === 'spotify' && <SpotifyAddForm allowedVariant={allowedVariant}/>}
      {sourceType === 'stream' && <StreamAddForm/>}
      {sourceType === 'file' && <FileAddForm/>}
    </>
  )
}