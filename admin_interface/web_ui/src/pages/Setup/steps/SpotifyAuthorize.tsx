import { useFormContext } from 'react-hook-form';
import { Box, Button, Chip, Stack, Typography } from '@mui/material';
import { Check } from '@mui/icons-material';
import { SetupFormValues } from '../Setup';

interface Props {
  onComplete: () => void;
  onBack: () => void;
}

export default function SpotifyAuthorize({ onComplete, onBack }: Props) {
  const { watch } = useFormContext<SetupFormValues>();
  const refreshToken = watch('spotifyRefreshToken');
  const isAuthorized = !!refreshToken;

  const handleAuth = () => {
    window.location.href = '/api/spotify/auth';
  };

  return (
    <div>
      <Typography variant="h5" gutterBottom>Authorise Spotify</Typography>
      <Typography variant="body1" sx={{ mb: 3 }}>
        Click the button below to authorise tinyghettobox to access your Spotify account.
        You will be redirected to Spotify and then back here.
      </Typography>

      <Box sx={{ textAlign: 'center', p: '32px' }}>
        <Button onClick={handleAuth} disabled={isAuthorized} variant="contained">
          {isAuthorized ? <><Check /> Authorised</> : 'Authorise with Spotify'}
        </Button>
      </Box>

      {isAuthorized && (
        <Box sx={{ textAlign: 'center', mb: 2 }}>
          <Chip color="success" label="Spotify connected" />
        </Box>
      )}

      <Stack direction="row" spacing={2} sx={{ mt: 2 }} justifyContent="flex-end">
        <Button onClick={onBack}>Back</Button>
        <Button variant="contained" onClick={onComplete} disabled={!isAuthorized}>
          Continue
        </Button>
      </Stack>
    </div>
  );
}
