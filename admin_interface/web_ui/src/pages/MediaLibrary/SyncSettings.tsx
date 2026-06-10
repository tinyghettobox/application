import {useEffect, useState} from 'react';
import {
  Box,
  Button,
  CircularProgress,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  TextField,
  Typography,
} from '@mui/material';
import {getSyncConfig, putSyncConfig} from '@/util/api';
import {CreateSyncConfig, SyncConfig, SORT_BY_LABELS, SORT_BY_OPTIONS, SYNC_INTERVAL_LABELS, SyncIntervalDays} from '@/types/sync';
import {notify} from '@/components/Notification';

interface Props {
  entryId: number;
}

const REGEX_SORT_STRATEGIES = new Set(['episode_regex_asc', 'episode_regex_desc']);
const INTERVAL_OPTIONS: SyncIntervalDays[] = [0, 1, 7, 30];

export default function SyncSettings({entryId}: Props) {
  const [syncConfig, setSyncConfig] = useState<SyncConfig | null | undefined>(undefined);
  const [form, setForm] = useState<CreateSyncConfig | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setSyncConfig(undefined);
    setForm(null);
    getSyncConfig(entryId).then(config => {
      setSyncConfig(config);
      if (config) {
        setForm({
          sortBy: config.sortBy,
          sortRegex: config.sortRegex,
          syncIntervalDays: config.syncIntervalDays,
        });
      }
    });
  }, [entryId]);

  if (syncConfig === undefined) {
    return <CircularProgress size={20}/>;
  }

  if (syncConfig === null || form === null) {
    return null;
  }

  const showRegex = REGEX_SORT_STRATEGIES.has(form.sortBy);

  const handleSave = async () => {
    setSaving(true);
    try {
      const updated = await putSyncConfig(entryId, {
        ...form,
        sortRegex: showRegex ? form.sortRegex : undefined,
      });
      setSyncConfig(updated);
      setForm({
        sortBy: updated.sortBy,
        sortRegex: updated.sortRegex,
        syncIntervalDays: updated.syncIntervalDays,
      });
      notify('Sync settings saved');
    } catch (e) {
      notify(`Failed to save sync settings: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Box sx={{mt: 4}}>
      <Typography variant="h6" sx={{mb: 2}}>Sync settings</Typography>
      <Box sx={{display: 'flex', flexDirection: 'column', gap: 2, maxWidth: 480}}>
        <FormControl fullWidth size="small">
          <InputLabel>Sort order</InputLabel>
          <Select
            label="Sort order"
            value={form.sortBy}
            onChange={e => setForm(f => f && ({...f, sortBy: e.target.value as any}))}
          >
            {SORT_BY_OPTIONS.map(option => (
              <MenuItem key={option} value={option}>{SORT_BY_LABELS[option]}</MenuItem>
            ))}
          </Select>
        </FormControl>

        {showRegex && (
          <TextField
            label="Episode regex"
            size="small"
            value={form.sortRegex ?? ''}
            placeholder="(\d+)"
            onChange={e => setForm(f => f && ({...f, sortRegex: e.target.value || undefined}))}
            helperText="Regex with one capture group to extract the episode number from the title"
          />
        )}

        <FormControl fullWidth size="small">
          <InputLabel>Sync interval</InputLabel>
          <Select
            label="Sync interval"
            value={form.syncIntervalDays}
            onChange={e => setForm(f => f && ({...f, syncIntervalDays: e.target.value as any}))}
          >
            {INTERVAL_OPTIONS.map(days => (
              <MenuItem key={days} value={days}>{SYNC_INTERVAL_LABELS[days]}</MenuItem>
            ))}
          </Select>
        </FormControl>

        <Box>
          <Button variant="contained" onClick={handleSave} disabled={saving}>
            {saving ? <CircularProgress size={20} sx={{mr: 1}}/> : null}
            Save
          </Button>
        </Box>
      </Box>
    </Box>
  );
}
