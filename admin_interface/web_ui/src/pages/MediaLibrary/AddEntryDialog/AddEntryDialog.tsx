import {Drawer} from "@mui/material";
import {AddEntryStateProvider} from "@/pages/MediaLibrary/AddEntryDialog/useAddEntryState";
import AddForm from "@/pages/MediaLibrary/AddEntryDialog/AddForm";
import {Variant} from "@db-models/Variant";
import {LibraryEntry} from "@db-models/LibraryEntry";

interface Props {
  parent: LibraryEntry;
  open: boolean;
  onClose: (submitted?: true) => void;
  allowedVariant?: Variant;
}

export default function AddEntryDialog({parent, open, onClose, allowedVariant}: Props) {
  return (
    <Drawer
      PaperProps={{sx: {width: '40%', padding: 4}}}
      variant="persistent"
      anchor="right"
      open={open}
      onClose={() => onClose()}
    >
      <AddEntryStateProvider parent={parent} onClose={onClose}>
        <AddForm allowedVariant={allowedVariant} />
      </AddEntryStateProvider>
    </Drawer>
  )
}