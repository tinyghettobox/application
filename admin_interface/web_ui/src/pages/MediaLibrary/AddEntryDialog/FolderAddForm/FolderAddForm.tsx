import {
  Box,
  Button,
  CircularProgress,
  Divider,
  FormControl,
  FormHelperText,
  FormLabel,
  Grid,
  IconButton,
  InputLabel,
  OutlinedInput,
  Stack,
  styled,
  Typography
} from "@mui/material";
import {Delete, Download, Upload} from "@mui/icons-material";
import {useAddEntryState} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";
import {ChangeEvent, useEffect, useState} from "react";
import {notify} from "@/components/Notification";
import {cropImage} from "@/pages/MediaLibrary/AddEntryDialog/helper";
import ImageEditor from "@/pages/MediaLibrary/AddEntryDialog/FolderAddForm/ImageEditor";

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

function isValidUrl(url: string) {
  return /(https?|s?ftp):\/\/[^.]+\.[^\/]+/.test(url);
}

export default function FolderAddForm() {
  const {entries, setEntries, getNextSortKey} = useAddEntryState();
  const [fetching, setFetching] = useState(false);
  const [fetchUrl, setFetchUrl] = useState('');
  const [uploading, setUploading] = useState(false);
  const [validUrl, setValidUrl] = useState(true);
  const folder = entries[0];

  useEffect(() => {
    if (!entries.length) {
      setEntries([{variant: 'folder', name: '', children: [], sortKey: getNextSortKey()}]);
    }
  }, [entries, setEntries]);

  const handleNameChange = (event: any) => {
    setEntries([{...entries[0], name: event.target.value}]);
  }

  const handleRemoveImage = () => {
    setEntries([{...entries[0], image: undefined}]);
  }

  const handleFetchUrlChange = (event: ChangeEvent<HTMLInputElement>) => {
    setFetchUrl(event.target.value);
    setValidUrl(!event.target.value || isValidUrl(event.target.value));
  }

  const handleFetch = async () => {
    setFetching(true);
    try {
      const query = new URLSearchParams({url: fetchUrl});
      const response = await fetch(`/api/image?${query}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch image: ${await response.text()}`);
      }
      const image = await cropImage(await response.blob(), 180, 180, 0.8);
      setEntries([{...entries[0], image: image}]);
    } catch (error) {
      notify('error', `${error}`, 8000);
    }
    setFetching(false);
  }

  const handleUpload = async (event: ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files?.length) {
      return;
    }
    setUploading(true);

    try {
      const image = await cropImage(event.target.files[0], 180, 180, 0.8);
      setEntries([{...entries[0], image}]);
    } catch (error) {
      notify('error', `${error}`, 5000);
    }

    setUploading(false);
  }

  return (
    <Box sx={{mt: 2}}>
      <FormControl fullWidth sx={{mb: 3}} size="small">
        <InputLabel>Name</InputLabel>
        <OutlinedInput label="Name" onChange={handleNameChange} value={folder?.name || ''}/>
      </FormControl>

      <FormControl fullWidth sx={{mb: 4}}>
        <FormLabel>Image</FormLabel>

        <Grid container alignItems={'center'}>
          <Grid item xs={3} textAlign={'center'}>
            <div style={{width: '134px', margin: '16px auto 0'}}>
              {/* TODO implement / fix image editor */}
              <ImageEditor folder={folder}/>
              {!!folder?.image &&
                <IconButton size="small" onClick={handleRemoveImage}><Delete/></IconButton>
              }
            </div>
          </Grid>
          <Grid item xs={9}>
            <Stack divider={<Divider><Typography variant="caption">or</Typography></Divider>} gap={2}>
              <Grid container>
                <Grid item xs={9}>
                  <FormControl fullWidth size="small" error={!validUrl}>
                    <InputLabel>Image URL</InputLabel>
                    <OutlinedInput
                      label="Image URL"
                      onChange={handleFetchUrlChange}
                      value={fetchUrl}
                    />
                    <FormHelperText>{!validUrl ? 'Please enter a valid url' : ' '}</FormHelperText>
                  </FormControl>
                </Grid>
                <Grid item xs={3} sx={{display: 'flex', justifyContent: 'center', alignItems: 'flex-start'}}>
                  <Button
                    variant="contained"
                    disabled={!fetchUrl || !validUrl || fetching}
                    onClick={handleFetch}
                    startIcon={fetching ? <CircularProgress size={16}/> : <Download/>}
                  >
                    Fetch
                  </Button>
                </Grid>
              </Grid>
              <Box sx={{textAlign: 'center', mt: 3}}>
                <Button
                  component="label"
                  variant="contained"
                  startIcon={uploading ? <CircularProgress size={16}/> : <Upload/>}
                  disabled={uploading}
                >
                  Upload image
                  <VisuallyHiddenInput type="file" name="image" accept="image/*" onChange={handleUpload}/>
                </Button>
              </Box>
              {/* TODO: Implement spotify based image search */}
            </Stack>
          </Grid>
        </Grid>
      </FormControl>
    </Box>
  )
}