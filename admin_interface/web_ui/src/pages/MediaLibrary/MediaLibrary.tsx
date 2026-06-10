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
import {useFolderSyncStatuses} from "@/pages/MediaLibrary/useFolderSyncStatuses";
import SyncSettings from "@/pages/MediaLibrary/SyncSettings";

export default function MediaLibrary() {
  const params = useParams();
  const entityId = params.id && parseInt(params.id) || 0;
  const {libraryEntry, loading, error, reloadLibraryEntry, deleteLibraryEntry, bulkUpdateLibraryEntries, setEntry, markPlayed} = useLibraryEntry(entityId);
  const [dialogOpen, setDialogOpen] = useState(false);
  const usedVariant = libraryEntry?.children?.map(child => child.variant)[0];
  const selection = useSelection(libraryEntry?.children ?? []);
  const {statuses: syncStatuses, handleTrigger: handleSyncTrigger} = useFolderSyncStatuses(entityId);
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
      <Grid container alignItems={"center"} sx={{mb: '16px', mt: {xs: '16px', md: '48px'}}}>
        <Grid item xs={10}>
          <Typography variant="h4">
            Media library
          </Typography>
        </Grid>
      </Grid>
      {!!libraryEntry ? (
        <Box sx={{pt: 2}}>
          <Box sx={{mb: 2}}>
            <Grid container spacing={1} sx={{mb: {xs: '24px', md: '48px'}}} alignItems="center" wrap="nowrap">
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
                <FolderAvatar folder={libraryEntry} sx={{width: {xs: '56px', md: '96px'}, height: {xs: '56px', md: '96px'}}}/>
              </Grid>
              <Grid item xs sx={{display: 'flex', flexDirection: 'column', justifyContent: 'center', minWidth: 0}}>
                <Typography variant="h5" sx={{mb: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'}}>{libraryEntry ? libraryEntry.name : ''}</Typography>
                <Stack direction={'row'} spacing={1} flexWrap="wrap" useFlexGap>
                  <Button variant="contained" onClick={handleOpenAddDialog} size="small">
                    <AddOutlined fontSize="small"/>
                    <Box component="span" sx={{display: {xs: 'none', sm: 'inline'}, ml: 0.5}}>Add entries</Box>
                  </Button>
                  {!!libraryEntry.children && (
                    <SortButton libraryEntries={libraryEntry.children} onSorted={handleSorted}/>
                  )}
                  <Button variant={'text'} disabled={selection.selectedItemIds.length === 0} onClick={handleMarkAsPlayed} size="small">
                    <CheckOutlined fontSize="small"/>
                    <Box component="span" sx={{display: {xs: 'none', sm: 'inline'}, ml: 0.5}}>Mark as {allSelectedPlayed ? 'not played' : 'played'}</Box>
                  </Button>
                </Stack>
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
                syncStatuses={syncStatuses}
                onTriggerSync={handleSyncTrigger}
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
          {libraryEntry.id !== 0 && (
            <SyncSettings entryId={libraryEntry.id!}/>
          )}
        </Box>

      ) : (
        loading ? <CircularProgress/> : <Typography variant="h5">Error: {error}</Typography>
      )}
    </div>
  )
}
