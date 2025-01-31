import {Speaker} from "@mui/icons-material";
import {Avatar, AvatarProps} from "@mui/material";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {arrayToBase64} from "@/util/base64";

interface Props {
  folder?: LibraryEntry;
  sx?: any;
  variant?: AvatarProps['variant'];
}

export default function FolderAvatar({folder, sx, variant}: Props) {
  let dataUri = '';
  if (folder?.image && folder.image.length > 0) {
    dataUri = `data:image/png;base64,${arrayToBase64(folder?.image)}`;
  }
  return (
    <Avatar src={dataUri} sx={sx} variant={variant}>
      <span>
        {folder?.name ? folder?.name.substring(0, 8).toUpperCase() : <Speaker />}
      </span>
    </Avatar>
  )
}