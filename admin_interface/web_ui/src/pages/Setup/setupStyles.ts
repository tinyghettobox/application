/**
 * Shared sx styles for the dark-glass Setup card.
 * Apply `glassSx` to a wrapping Box around form inputs so MUI
 * outlined inputs/selects render with light-on-dark colours.
 */
export const glassSx = {
  // Outlined input border + label
  '& .MuiOutlinedInput-root': {
    color: 'white',
    '& fieldset': { borderColor: 'rgba(255,255,255,0.25)' },
    '&:hover fieldset': { borderColor: 'rgba(255,255,255,0.55)' },
    '&.Mui-focused fieldset': { borderColor: '#e94560' },
  },
  '& .MuiInputLabel-root': { color: 'rgba(255,255,255,0.6)' },
  '& .MuiInputLabel-root.Mui-focused': { color: '#e94560' },
  '& .MuiFormHelperText-root': { color: 'rgba(255,255,255,0.5)' },
  '& .MuiFormHelperText-root.Mui-error': { color: '#f28b82' },
  '& .MuiSelect-icon': { color: 'rgba(255,255,255,0.5)' },
  '& .MuiInputBase-input': { color: 'white' },
  // Alert backgrounds look better semi-transparent on dark
  '& .MuiAlert-root': { backdropFilter: 'blur(8px)' },
} as const;

export const stepHeadingSx = {
  color: 'white',
  fontWeight: 700,
  mb: 1,
} as const;

export const stepBodySx = {
  color: 'rgba(255,255,255,0.7)',
  mb: 3,
} as const;

export const backButtonSx = {
  color: 'rgba(255,255,255,0.7)',
  borderColor: 'rgba(255,255,255,0.25)',
  '&:hover': { borderColor: 'rgba(255,255,255,0.6)', background: 'rgba(255,255,255,0.05)' },
} as const;

export const primaryButtonSx = {
  background: 'linear-gradient(135deg, #e94560, #c0392b)',
  '&:hover': { background: 'linear-gradient(135deg, #f05570, #e74c3c)' },
  '&.Mui-disabled': { opacity: 0.4, background: 'linear-gradient(135deg, #e94560, #c0392b)', color: 'white' },
  color: 'white',
  boxShadow: '0 4px 15px rgba(233,69,96,0.35)',
} as const;
