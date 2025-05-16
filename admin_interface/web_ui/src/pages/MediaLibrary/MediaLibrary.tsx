import {useState} from "react";
import {Box, Breadcrumbs, Button, CircularProgress, Grid, Stack, Typography} from "@mui/material";
import FolderList from "./FolderList";
import TrackList from "./TrackList";
import {AddOutlined, ArrowLeft, CheckOutlined, Home, WestOutlined} from "@mui/icons-material";
import FolderAvatar from "@/components/FolderAvatar";
import {useLibraryEntry} from "@/pages/MediaLibrary/useLibraryEntry";
import SortButton from "@/pages/MediaLibrary/SortButton";
import AddEntryDialog from "@/pages/MediaLibrary/AddEntryDialog/AddEntryDialog";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {useParams, Link} from "react-router-dom";
import {notify} from "@/components/Notification";
import useSelection from "./useSelection";

export default function MediaLibrary() {
  const params = useParams();
  const entityId = params.id && parseInt(params.id) || 0;
  const {libraryEntry, loading, error, reloadLibraryEntry, deleteLibraryEntry, bulkUpdateLibraryEntries, setEntry, markPlayed} = useLibraryEntry(entityId);
  const [dialogOpen, setDialogOpen] = useState(false);
  const usedVariant = libraryEntry?.children?.map(child => child.variant)[0];
  const selection = useSelection(libraryEntry?.children ?? []);
  const allSelectedPlayed = libraryEntry?.children
    ?.filter(child => selection.selectedItemIds.includes(child.id!))
    .some(child => child.playedAt);

  const handleOpenAddDialog = () => {
    setDialogOpen(true)
  }

  const handleCloseAddDialog = (submitted?: true) => {
    if (submitted) {
      reloadLibraryEntry();
    }
    setDialogOpen(false);
  }

  const handleDelete = async (entry: LibraryEntry) => {
    if (entry.id === null || entry.id === undefined) {
      return;
    }
    if (!confirm(`Are you sure you want to delete ${entry.name}?`)) {
      return;
    }

    await deleteLibraryEntry(entry.id);
  }

  const handleSortEnd = async (items: LibraryEntry[]) => {
    const sortedItems = items.map((item, index) => {
      const entry = libraryEntry?.children?.find(entry => entry.id === item.id) as LibraryEntry;
      entry.sortKey = index;
      return entry;
    });
    // We have to update the state already to not wait for network and risk flickering
    if (libraryEntry) {
      libraryEntry.children = sortedItems;
      setEntry(libraryEntry);
    }

    await bulkUpdateLibraryEntries(sortedItems.map(item => ({id: item.id, sortKey: item.sortKey})));
    selection.clearSelection();
  }

  const handleSorted = async (libraryEntries: LibraryEntry[]) => {
    await bulkUpdateLibraryEntries(libraryEntries.map(item => ({id: item.id, sortKey: item.sortKey})));
  }

  const handleMarkAsPlayed = async (event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    await markPlayed(selection.selectedItemIds, allSelectedPlayed ? null : new Date().toISOString());

    selection.clearSelection();
  }

  return (
    <div>
      <Grid container alignItems={"center"} sx={{mb: '24px', mt: '48px'}}>
        <Grid item xs={10}>
          <Typography variant="h4">
            Media library
          </Typography>
        </Grid>
      </Grid>
      {!!libraryEntry ? (
        <Box sx={{pt: 2}}>
          <Box sx={{mb: 2}}>
            <Grid container spacing={2} sx={{mb: '48px'}}>
              {libraryEntry.id !== 0 && (
                <Grid
                  item
                  xs={'auto'}
                  sx={{display: 'flex', flexDirection: 'column', justifyContent: 'center'}}
                >
                  <Link to={`/mediaLibrary/${libraryEntry?.parentId || ''}`} color="inherit">
                    <WestOutlined/>
                  </Link>
                </Grid>
              )}
              <Grid item xs={'auto'}>
                <FolderAvatar folder={libraryEntry} sx={{width: '96px', height: '96px'}}/>
              </Grid>
              <Grid item xs sx={{display: 'flex', flexDirection: 'column', justifyContent: 'center'}}>
                <Typography variant="h5" sx={{mb: 1}}>{libraryEntry ? libraryEntry.name : ''}</Typography>
                <Grid container gap={2}>
                  <Stack direction={'row'} spacing={2}>
                    <Button variant="contained" onClick={handleOpenAddDialog}>
                      <AddOutlined/>&nbsp;
                      Add entries
                    </Button>
                    {!!libraryEntry.children && (
                      <SortButton libraryEntries={libraryEntry.children} onSorted={handleSorted}/>
                    )}
                    <Button variant={'text'} disabled={selection.selectedItemIds.length === 0} onClick={handleMarkAsPlayed}>
                      <CheckOutlined/>&nbsp;
                      Mark as {allSelectedPlayed ? 'not played' : 'played'}
                    </Button>
                  </Stack>
                </Grid>
              </Grid>
            </Grid>
          </Box>
          {libraryEntry.children && (
            libraryEntry.children.some(entry => entry.variant === 'folder') ? (
              <FolderList
                folders={libraryEntry.children.filter(entry => entry.variant === 'folder')}
                onSortEnd={handleSortEnd}
                onDelete={handleDelete}
                selectedItemIds={selection.selectedItemIds}
                onSelect={selection.handleSelect}
              />
            ) : (
              <TrackList
                tracks={libraryEntry.children.filter(entry => entry.variant !== 'folder')}
                onSortEnd={handleSortEnd}
                onDelete={handleDelete}
                selectedItemIds={selection.selectedItemIds}
                onSelect={selection.handleSelect}
              />
            )
          )}
          {!!libraryEntry &&
            <AddEntryDialog parent={libraryEntry} open={dialogOpen} onClose={handleCloseAddDialog} allowedVariant={usedVariant}/>
          }
        </Box>

      ) : (
        loading ? <CircularProgress/> : <Typography variant="h5">Error: {error}</Typography>
      )}
    </div>
  )
}
