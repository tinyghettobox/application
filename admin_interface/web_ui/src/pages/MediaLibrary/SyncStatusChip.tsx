import {useState} from 'react';
import {Box, Chip, CircularProgress, IconButton, Tooltip} from '@mui/material';
import {Sync, SyncProblem, CheckCircleOutline, HourglassEmpty} from '@mui/icons-material';
import {postTriggerSync} from '@/util/api';
import {SyncStatus, SyncStatusKind} from '@/types/sync';

interface Props {
  status: SyncStatus | null;
  /** Called after a manual sync trigger so the parent can refresh statuses. */
  onRefresh: () => void;
}

export default function SyncStatusChip({status, onRefresh}: Props) {
  const [triggering, setTriggering] = useState(false);

  if (!status) return null;

  const handleTrigger = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setTriggering(true);
    try {
      await postTriggerSync(status.libraryEntryId);
      onRefresh();
    } finally {
      setTriggering(false);
    }
  };

  const isActive = status.status === 'pending' || status.status === 'running';

  return (
    <Box sx={{display: 'flex', alignItems: 'center', gap: 0.5, mt: 0.5}}>
      <StatusChip status={status}/>
      {!isActive && (
        <Tooltip title="Sync now">
          <span>
            <IconButton
              size="small"
              onClick={handleTrigger}
              disabled={triggering}
              sx={{p: 0.25}}
            >
              {triggering ? <CircularProgress size={16}/> : <Sync fontSize="small"/>}
            </IconButton>
          </span>
        </Tooltip>
      )}
    </Box>
  );
}

function StatusChip({status}: {status: SyncStatus}) {
  switch (status.status as SyncStatusKind) {
    case 'pending':
      return (
        <Chip
          icon={<HourglassEmpty fontSize="small"/>}
          label="Sync pending"
          size="small"
          color="default"
          variant="outlined"
        />
      );
    case 'running': {
      const label = status.itemsTotal != null
        ? `Syncing… ${status.itemsDone} / ${status.itemsTotal}`
        : `Syncing… ${status.itemsDone}`;
      return (
        <Chip
          icon={<CircularProgress size={14}/>}
          label={label}
          size="small"
          color="primary"
          variant="outlined"
        />
      );
    }
    case 'done':
      return (
        <Chip
          icon={<CheckCircleOutline fontSize="small"/>}
          label={`Synced${status.syncedAt ? ` ${formatRelative(status.syncedAt)}` : ''}`}
          size="small"
          color="success"
          variant="outlined"
        />
      );
    case 'error':
      return (
        <Tooltip title={status.errorMessage ?? 'Unknown error'}>
          <Chip
            icon={<SyncProblem fontSize="small"/>}
            label="Sync error"
            size="small"
            color="error"
            variant="outlined"
          />
        </Tooltip>
      );
  }
}

function formatRelative(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime();
  const minutes = Math.floor(diff / 60_000);
  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}
