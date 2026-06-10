import { useState } from 'react';
import { Controller, useFormContext } from 'react-hook-form';
import {
  Button,
  FormControl,
  Grid,
  InputLabel,
  OutlinedInput,
  Stack,
  Typography,
} from '@mui/material';
import { getSpotifyConfig, putSpotifyConfig } from '@/util/api';
import { notify } from '@/components/Notification';
import { SetupFormValues } from '../Setup';

interface Props {
  onComplete: () => void;
  onBack: () => void;
}

export default function SpotifyCredentials({ onComplete, onBack }: Props) {
  const { control, getValues, setValue } = useFormContext<SetupFormValues>();
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      const { spotifyClientId, spotifySecretKey } = getValues();
      const saved = await putSpotifyConfig({ clientId: spotifyClientId, secretKey: spotifySecretKey } as any);
      setValue('spotifyClientId', (saved as any).clientId ?? spotifyClientId);
      notify('success', 'Credentials saved', 2000);
      onComplete();
    } catch (e) {
      notify('error', `Failed to save: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div>
      <Typography variant="h5" gutterBottom>Spotify credentials</Typography>
      <Typography variant="body1" sx={{ mb: 3 }}>
        Create a developer application at{' '}
        <a href="https://developer.spotify.com" target="_blank" rel="noreferrer">
          developer.spotify.com
        </a>{' '}
        and enter the Client ID and Secret Key below. Set the redirect URI in the Spotify app to{' '}
        <code>http://tinyghettobox.local/api/spotify/auth/callback</code>.
      </Typography>

      <Grid container spacing={2} sx={{ maxWidth: 480 }}>
        <Grid item xs={6}>
          <Controller
            name="spotifyClientId"
            control={control}
            rules={{ required: true }}
            render={({ field, fieldState }) => (
              <FormControl fullWidth error={fieldState.invalid}>
                <InputLabel>Client ID</InputLabel>
                <OutlinedInput {...field} label="Client ID" />
              </FormControl>
            )}
          />
        </Grid>
        <Grid item xs={6}>
          <Controller
            name="spotifySecretKey"
            control={control}
            rules={{ required: true }}
            render={({ field, fieldState }) => (
              <FormControl fullWidth error={fieldState.invalid}>
                <InputLabel>Secret Key</InputLabel>
                <OutlinedInput {...field} label="Secret Key" />
              </FormControl>
            )}
          />
        </Grid>
      </Grid>

      <Stack direction="row" spacing={2} sx={{ mt: 3 }}>
        <Button onClick={onBack}>Back</Button>
        <Button variant="contained" onClick={handleSave} disabled={saving}>
          Save &amp; Continue
        </Button>
      </Stack>
    </div>
  );
}
