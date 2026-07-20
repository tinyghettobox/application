import { useState } from 'react';
import { Controller, useFormContext } from 'react-hook-form';
import {
  Alert,
  Box,
  Button,
  CircularProgress,
  FormControl,
  FormHelperText,
  InputLabel,
  List,
  ListItemButton,
  ListItemSecondaryAction,
  ListItemText,
  OutlinedInput,
  Stack,
  Typography,
} from '@mui/material';
import NetworkWifiIcon from '@mui/icons-material/NetworkWifi';
import WifiLockIcon from '@mui/icons-material/WifiLock';
import { getWifiNetworks, getWifiStatus, postWifiConnect, WifiNetwork, WifiStatusResponse } from '@/util/api';
import { SetupFormValues } from '../Setup';

interface Props {
  onComplete: () => void;
}

function signalBars(dBm: number): string {
  if (dBm >= -50) return '▂▄▆█';
  if (dBm >= -65) return '▂▄▆·';
  if (dBm >= -75) return '▂▄··';
  return '▂···';
}

function securityLabel(security: WifiNetwork['security']): string {
  switch (security) {
    case 'wpa3': return 'WPA3';
    case 'wpa2': return 'WPA2';
    case 'wpa':  return 'WPA';
    case 'wep':  return 'WEP';
    default:     return 'Open';
  }
}

export default function WifiSetup({ onComplete }: Props) {
  const { control, getValues, setValue } = useFormContext<SetupFormValues>();
  const [networks, setNetworks] = useState<WifiNetwork[] | null>(null);
  const [scanning, setScanning] = useState(false);
  const [selectedSsid, setSelectedSsid] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const handleScan = async () => {
    setScanning(true);
    setError(null);
    setNetworks(null);
    setStatusMessage('Scanning… the access point will be briefly unavailable.');
    try {
      const found = await getWifiNetworks();
      setNetworks(found);
      setStatusMessage(null);
      if (found.length === 0) {
        setError('No networks found. Make sure you are within range and try again.');
      }
    } catch {
      setError('Scan failed. The device may have temporarily lost connectivity — please reload the page and try again.');
      setStatusMessage(null);
    } finally {
      setScanning(false);
    }
  };

  const handleSelectNetwork = (network: WifiNetwork) => {
    setSelectedSsid(network.ssid);
    setValue('ssid', network.ssid);
    setValue('security', network.security === 'nopass' ? 'nopass' : network.security);
    setError(null);
  };

  const handleConnect = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    const { ssid, wifiPassword, security } = getValues();
    if (!ssid) return;

    setConnecting(true);
    setError(null);
    setStatusMessage('Sending connection request…');

    try {
      await postWifiConnect({ ssid, password: wifiPassword, security });
    } catch {
      // The request may fail if the network drops mid-POST — that's expected.
    }

    setStatusMessage('Waiting for wifi connection… the page may briefly become unreachable while the access point switches off.');

    const poll = async (): Promise<void> => {
      let res: WifiStatusResponse;
      try {
        res = await getWifiStatus();
      } catch {
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
        await delay(2000);
        return poll();
      }
    };

    await poll();
  };

  const ssid = getValues('ssid');
  const security = getValues('security');
  const needsPassword = security !== 'nopass';

  return (
    <div>
      <Typography variant="h5" gutterBottom>WiFi Setup</Typography>
      <Typography variant="body1" sx={{ mb: 2 }}>
        Connect tinyghettobox to your home wifi network. After connecting, the access point
        will be disabled and the device will be accessible at{' '}
        <strong>http://tinyghettobox.local</strong>.
      </Typography>

      <Button
        variant="outlined"
        onClick={handleScan}
        disabled={scanning || connecting}
        startIcon={scanning ? <CircularProgress size={16} /> : <NetworkWifiIcon />}
        sx={{ mb: 2 }}
      >
        {scanning ? 'Scanning…' : networks === null ? 'Scan for networks' : 'Scan again'}
      </Button>

      {networks !== null && networks.length > 0 && (
        <Box sx={{ maxWidth: 480, border: 1, borderColor: 'divider', borderRadius: 1, mb: 2, maxHeight: 260, overflowY: 'auto' }}>
          <List dense disablePadding>
            {networks.map((n) => (
              <ListItemButton
                key={n.ssid}
                selected={selectedSsid === n.ssid}
                onClick={() => handleSelectNetwork(n)}
              >
                <ListItemText
                  primary={n.ssid}
                  secondary={securityLabel(n.security)}
                />
                <ListItemSecondaryAction>
                  {n.security !== 'nopass' && (
                    <WifiLockIcon fontSize="small" sx={{ mr: 0.5, verticalAlign: 'middle', opacity: 0.6 }} />
                  )}
                  <Typography variant="caption" sx={{ fontFamily: 'monospace' }}>
                    {signalBars(n.signal)}
                  </Typography>
                </ListItemSecondaryAction>
              </ListItemButton>
            ))}
          </List>
        </Box>
      )}

      {selectedSsid && (
        <Stack spacing={2} sx={{ maxWidth: 480, mb: 2 }}>
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

          {needsPassword && (
            <Controller
              name="wifiPassword"
              control={control}
              render={({ field }) => (
                <FormControl fullWidth>
                  <InputLabel>Password</InputLabel>
                  <OutlinedInput {...field} label="Password" type="password" autoFocus />
                </FormControl>
              )}
            />
          )}
        </Stack>
      )}

      {!networks && !scanning && (
        <Box sx={{ maxWidth: 480 }}>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
            Or enter the network details manually:
          </Typography>
          <Stack spacing={2}>
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
          </Stack>
        </Box>
      )}

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

      {(selectedSsid || (!networks && !scanning)) && (
        <Stack direction="row" spacing={2} sx={{ mt: 3 }}>
          <Button
            variant="contained"
            onClick={handleConnect}
            disabled={connecting || !ssid}
            startIcon={connecting ? <CircularProgress size={16} /> : undefined}
          >
            {connecting ? 'Connecting…' : 'Connect'}
          </Button>
        </Stack>
      )}
    </div>
  );
}

function delay(ms: number) {
  return new Promise<void>(resolve => setTimeout(resolve, ms));
}
