import {
  Box,
  Collapse,
  FormControl,
  FormHelperText,
  Icon,
  InputLabel,
  MenuItem,
  Select,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material';
import {
  CreateSyncConfig,
  SORT_BY_LABELS,
  SORT_BY_OPTIONS,
  SortBy,
  SYNC_INTERVAL_LABELS,
  SyncIntervalDays,
} from '@/types/sync';
import { QuestionMark } from '@mui/icons-material';

interface Props {
  value: CreateSyncConfig;
  onChange: (config: CreateSyncConfig) => void;
}

const INTERVAL_OPTIONS: SyncIntervalDays[] = [1, 7, 30, 0];

export default function SyncConfigSection({value, onChange}: Props) {
  const needsRegex =
    value.sortBy === 'episode_regex_asc' || value.sortBy === 'episode_regex_desc';

  return (
    <Box sx={{pl: 1, pr: 1}}>
      <Typography variant="subtitle2" sx={{mb: 1}}>
        Sync settings
        <Tooltip title="Configure how this library entry should be kept in sync with its Spotify source. Syncing is done by periodically checking the Spotify API for changed children and adding or removing children from the library. You can also trigger a manual sync at any time from the library view.">
          <QuestionMark fontSize='inherit' color='info' sx={{ml: 0.5, border: '1px solid', borderRadius: '50%', p: 0.25, verticalAlign: 'middle'}}/>
        </Tooltip>
      </Typography>

      <FormControl fullWidth size="small" sx={{mb: 2, mt: 2}}>
        <InputLabel id="sort-by-label">Sort children by</InputLabel>
        <Select
          labelId="sort-by-label"
          label="Sort children by"
          value={value.sortBy}
          onChange={(e) => onChange({...value, sortBy: e.target.value as SortBy})}
        >
          {SORT_BY_OPTIONS.map((opt) => (
            <MenuItem key={opt} value={opt}>
              {SORT_BY_LABELS[opt]}
            </MenuItem>
          ))}
        </Select>
      </FormControl>

      <Collapse in={needsRegex}>
        <TextField
          fullWidth
          size="small"
          label="Episode number regex"
          placeholder="e.g. (\d+)"
          value={value.sortRegex ?? ''}
          onChange={(e) => onChange({...value, sortRegex: e.target.value || undefined})}
          sx={{mb: 2}}
          helperText="Capture group 1 is used as the episode number"
        />
      </Collapse>

      <FormControl fullWidth size="small">
        <InputLabel id="sync-interval-label">Sync interval</InputLabel>
        <Select
          labelId="sync-interval-label"
          label="Sync interval"
          value={value.syncIntervalDays}
          onChange={(e) =>
            onChange({...value, syncIntervalDays: Number(e.target.value)})
          }
        >
          {INTERVAL_OPTIONS.map((days) => (
            <MenuItem key={days} value={days}>
              {SYNC_INTERVAL_LABELS[days]}
            </MenuItem>
          ))}
        </Select>
        <FormHelperText>
          How often to check for new content on Spotify
        </FormHelperText>
      </FormControl>
    </Box>
  );
}
