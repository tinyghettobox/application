'use client';
import {
  Button,
  CircularProgress,
  Divider,
  FormControl,
  Grid,
  InputLabel,
  MenuItem,
  OutlinedInput,
  Select, Typography
} from "@mui/material";
import {useState} from "react";
import SpotifyPlaylist from "./SpotifyPlaylist";
import {Controller, useForm} from "react-hook-form";
import {SearchOutlined, EastOutlined} from "@mui/icons-material";
import {searchResultToLibraryEntry} from "./helper";
import {Variant} from "@db-models/Variant";
import {LibraryEntry} from "@db-models/LibraryEntry";
import styles from "@/pages/MediaLibrary/AddEntryDialog/SpotifyAddForm/SpotifyAddForm.module.scss";
import SpotifyItem from "@/pages/MediaLibrary/AddEntryDialog/SpotifyAddForm/SpotifyItem";
import {useAddEntryState} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";

interface Props {
  allowedVariant?: Variant;
}

const SEARCH_TYPES = ['artist', 'album', 'playlist', 'track', 'show', 'episode'];

export default function SpotifyAddForm({allowedVariant}: Props) {
  const {getNextSortKey} = useAddEntryState();
  const [searchResult, setSearchResult] = useState({loading: false, data: [] as LibraryEntry[]});
  const {
    control,
    handleSubmit,
    watch,
    formState
  } = useForm<{ search: string, searchType: string }>({
    mode: "onTouched",
    defaultValues: {search: '', searchType: SEARCH_TYPES[0]}
  });

  const search = async (formData: { search: string, searchType: string }) => {
    setSearchResult({loading: true, data: []})

    const params = new URLSearchParams({search: formData.search, search_type: formData.searchType})
    const response = await fetch('/api/spotify/search?' + params);

    if (response.status !== 200) {
      alert(`Search failed: ${await response.text()}`);
      setSearchResult({loading: false, data: []})
      return;
    }

    const data = await response.json();
    const searchResult = data[`${formData.searchType}s`];

    if (!searchResult || !Array.isArray(searchResult.items)) {
      alert(`Search did not return searchType: ${JSON.stringify(data)}`);
      setSearchResult({loading: false, data: []})
      return;
    }

    const entries: LibraryEntry[] = [];
    for (const searchResultItem of searchResult.items) {
      entries.push(await searchResultToLibraryEntry(searchResultItem, formData.searchType, getNextSortKey()));
    }

    setSearchResult({loading: false, data: entries});
  }

  return (
    <form onSubmit={handleSubmit(search)}>
      <Grid container gap={1}>
        <Grid item xs={7}>
          <Grid container sx={{mt: '0px', mb: '24px'}} spacing={2}>
            <Grid item xs={7}>
              <Controller
                name={'search'}
                rules={{required: true}}
                control={control}
                render={({field, fieldState}) =>
                  <FormControl fullWidth size="small" error={fieldState.invalid}>
                    <InputLabel id={"search-label"}>Search</InputLabel>
                    <OutlinedInput label={"Search"} {...field} />
                  </FormControl>
                }
              />
            </Grid>
            <Grid item xs={3}>
              <Controller
                name={'searchType'}
                control={control}
                render={({field: {onChange, ...field}, fieldState}) =>
                  <FormControl fullWidth size="small" error={fieldState.invalid}>
                    <InputLabel id={'search-type'}>Search type</InputLabel>
                    <Select
                      variant="outlined"
                      labelId="search-type"
                      label={'Search type'}
                      onChange={(event) => onChange(event.target.value)}
                      {...field}
                    >
                      {SEARCH_TYPES.map(type =>
                        <MenuItem key={type} value={type.toLowerCase()}>{type}</MenuItem>
                      )}
                    </Select>
                  </FormControl>
                }
              />
            </Grid>
            <Grid item xs={2}>
              <Button
                variant="outlined"
                type={'submit'}
                disabled={!watch('search')}
                sx={{height: '100%'}}
              ><SearchOutlined/></Button>
            </Grid>
          </Grid>

          {searchResult.loading ?
            <CircularProgress/>
            :
            searchResult.data.length !== 0 ?
              <div className={styles.resultList}>
                <Typography variant="h5" className={styles.title}>{watch('searchType')}s</Typography>
                {searchResult.data.map(item => <SpotifyItem key={item.name} entry={item}
                                                            allowedVariant={allowedVariant}/>)}
              </div>
              :
              formState.isSubmitted && <Typography>No search result yet</Typography>
          }
        </Grid>
        <Divider orientation="vertical" textAlign={'left'} flexItem>
          <EastOutlined/>
        </Divider>
        <Grid item xs={4}>
          <SpotifyPlaylist/>
        </Grid>
      </Grid>
    </form>
  )
}