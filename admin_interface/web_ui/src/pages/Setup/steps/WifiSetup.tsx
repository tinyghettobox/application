import { useState } from 'react';
import { Controller, useFormContext } from 'react-hook-form';
import {
  Alert,
  Box,
  Button,
  CircularProgress,
  FormControl,
  FormHelperText,
  Grid,
  InputLabel,
  MenuItem,
  OutlinedInput,
  Select,
  Stack,
  Typography,
} from '@mui/material';
import { getWifiStatus, postWifiConnect, WifiStatusResponse } from '@/util/api';
import { SetupFormValues } from '../Setup';

interface Props {
  onComplete: () => void;
}

export default function WifiSetup({ onComplete }: Props) {
  const { control, getValues } = useFormContext<SetupFormValues>();
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const handleConnect = async () => {
    const { ssid, wifiPassword, security } = getValues();
    if (!ssid) return;

    setConnecting(true);
    setError(null);
    setStatusMessage('Sending connection request…');

    try {
      await postWifiConnect({ ssid, password: wifiPassword, security });
    } catch {
      // The request may fail if the network drops mid-POST — that's expected.
      // Continue polling regardless.
    }

    setStatusMessage('Waiting for wifi connection… the page may briefly become unreachable while the access point switches off.');

    // Poll until connected or failed.
    const poll = async (): Promise<void> => {
      let res: WifiStatusResponse;
      try {
        res = await getWifiStatus();
      } catch {
        // Network temporarily unreachable — device is probably switching networks.
        // Keep polling.
        await delay(2000);
        return poll();
      }

      if (res.status === 'connected') {
        setConnecting(false);
        setStatusMessage(null);
        onComplete();
      } else if (res.status === 'failed') {
        setConnecting(false);
        setError(res.error ?? 'Connection failed. Check your credentials and try again.');
        setStatusMessage(null);
      } else {
        // still connecting
        await delay(2000);
        return poll();
      }
    };

    await poll();
  };

  return (
    <div>
      <Typography variant="h5" gutterBottom>WiFi Setup</Typography>
      <Typography variant="body1" sx={{ mb: 3 }}>
        Connect tinyghettobox to your home wifi network. After connecting, the access point
        will be disabled and the device will be accessible at{' '}
        <strong>http://tinyghettobox.local</strong>.
      </Typography>

      <Grid container spacing={2} sx={{ maxWidth: 480 }}>
        <Grid item xs={12}>
          <Controller
            name="ssid"
            control={control}
            rules={{ required: 'SSID is required' }}
            render={({ field, fieldState }) => (
              <FormControl fullWidth error={!!fieldState.error}>
                <InputLabel>Network name (SSID)</InputLabel>
                <OutlinedInput {...field} label="Network name (SSID)" />
                {fieldState.error && <FormHelperText>{fieldState.error.message}</FormHelperText>}
              </FormControl>
            )}
          />
        </Grid>

        <Grid item xs={8}>
          <Controller
            name="wifiPassword"
            control={control}
            render={({ field }) => (
              <FormControl fullWidth>
                <InputLabel>Password</InputLabel>
                <OutlinedInput {...field} label="Password" type="password" />
              </FormControl>
            )}
          />
        </Grid>

        <Grid item xs={4}>
          <Controller
            name="security"
            control={control}
            render={({ field }) => (
              <FormControl fullWidth>
                <InputLabel>Security</InputLabel>
                <Select {...field} label="Security">
                  <MenuItem value="WPA">WPA/WPA2</MenuItem>
                  <MenuItem value="WEP">WEP</MenuItem>
                  <MenuItem value="nopass">None</MenuItem>
                </Select>
              </FormControl>
            )}
          />
        </Grid>
      </Grid>

      {error && (
        <Alert severity="error" sx={{ mt: 2, maxWidth: 480 }}>
          {error}
        </Alert>
      )}

      {statusMessage && (
        <Alert severity="info" sx={{ mt: 2, maxWidth: 480 }}>
          {statusMessage}
        </Alert>
      )}

      <Stack direction="row" spacing={2} sx={{ mt: 3 }}>
        <Button
          variant="contained"
          onClick={handleConnect}
          disabled={connecting}
          startIcon={connecting ? <CircularProgress size={16} /> : undefined}
        >
          {connecting ? 'Connecting…' : 'Connect'}
        </Button>
      </Stack>
    </div>
  );
}

function delay(ms: number) {
  return new Promise<void>(resolve => setTimeout(resolve, ms));
}
