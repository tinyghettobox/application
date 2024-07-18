import {Speaker} from "@mui/icons-material";
import {Avatar} from "@mui/material";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {arrayToBase64} from "@/util/base64";

interface Props {
  folder?: LibraryEntry;
  sx?: any;
}

export default function FolderAvatar({folder, sx}: Props) {
  let dataUri = '';
  if (folder?.image && folder.image.length > 0) {
    dataUri = `data:image/png;base64,${arrayToBase64(folder?.image)}`;
  }
  return (
    <Avatar src={dataUri} sx={sx}>
      <span>
        {folder?.name ? folder?.name.substring(0, 8).toUpperCase() : <Speaker />}
      </span>
    </Avatar>
  )
}