import { useState } from 'react';
import { Box, Button, CircularProgress, Typography } from '@mui/material';
import { CheckCircleOutline } from '@mui/icons-material';
import { useNavigate } from 'react-router-dom';
import { putSystemConfig } from '@/util/api';
import { notify } from '@/components/Notification';

export default function Complete() {
  const navigate = useNavigate();
  const [saving, setSaving] = useState(false);

  const handleGoToLibrary = async () => {
    setSaving(true);
    try {
      await putSystemConfig({ setupComplete: true } as any);
      navigate('/mediaLibrary');
    } catch (e) {
      notify('error', `Could not complete setup: ${e}`);
      setSaving(false);
    }
  };

  return (
    <Box sx={{ textAlign: 'center', pt: 4 }}>
      <CheckCircleOutline sx={{ fontSize: 80, color: 'success.main', mb: 2 }} />
      <Typography variant="h4" gutterBottom>
        Setup complete!
      </Typography>
      <Typography variant="body1" sx={{ mb: 4, maxWidth: 480, mx: 'auto' }}>
        tinyghettobox is ready. You can now add music to your library and start listening.
      </Typography>
      <Button
        variant="contained"
        size="large"
        onClick={handleGoToLibrary}
        disabled={saving}
        startIcon={saving ? <CircularProgress size={18} /> : undefined}
      >
        Go to Library
      </Button>
    </Box>
  );
}
