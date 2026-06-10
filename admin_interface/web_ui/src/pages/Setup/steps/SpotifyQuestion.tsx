import { useFormContext } from 'react-hook-form';
import { Box, Button, Stack, Typography } from '@mui/material';
import { SetupFormValues } from '../Setup';

interface Props {
  onYes: () => void;
  onNo: () => void;
}

export default function SpotifyQuestion({ onYes, onNo }: Props) {
  const { setValue } = useFormContext<SetupFormValues>();

  const handleYes = () => {
    setValue('useSpotify', true);
    onYes();
  };

  const handleNo = () => {
    setValue('useSpotify', false);
    onNo();
  };

  return (
    <Box>
      <Typography variant="h5" gutterBottom>Spotify</Typography>
      <Typography variant="body1" sx={{ mb: 4 }}>
        Would you like to use Spotify with tinyghettobox? You can always configure this later
        in the admin settings.
      </Typography>
      <Stack direction="row" spacing={2}>
        <Button variant="contained" onClick={handleYes}>
          Yes, set up Spotify
        </Button>
        <Button variant="outlined" onClick={handleNo}>
          Skip for now
        </Button>
      </Stack>
    </Box>
  );
}
