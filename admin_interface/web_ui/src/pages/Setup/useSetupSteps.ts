import { useCallback, useMemo } from 'react';
import { useFormContext } from 'react-hook-form';
import { useParams } from 'react-router-dom';
import { SetupFormValues } from './Setup';

export const STEP_WIFI = 0;
export const STEP_SPOTIFY_QUESTION = 1;
export const STEP_SPOTIFY_CREDENTIALS = 2;
export const STEP_SPOTIFY_AUTHORIZE = 3;
export const STEP_COMPLETE = 4;

export function useSetupSteps() {
  const { step } = useParams<{ step?: string }>();
  const { watch } = useFormContext<SetupFormValues>();

  const [ssid, useSpotify, spotifyClientId, spotifyRefreshToken] = watch([
    'ssid',
    'useSpotify',
    'spotifyClientId',
    'spotifyRefreshToken',
  ]);

  // Derive active step from URL param; default to 0.
  const activeStep = parseInt(step ?? '0') || 0;

  const steps = useMemo(() => [
    { label: 'WiFi', completed: !!ssid },
    { label: 'Spotify?', completed: useSpotify !== null },
    { label: 'Spotify credentials', completed: !!spotifyClientId, optional: true, skip: useSpotify === false },
    { label: 'Spotify authorise', completed: !!spotifyRefreshToken, optional: true, skip: useSpotify === false },
    { label: 'Complete', completed: false },
  ], [ssid, useSpotify, spotifyClientId, spotifyRefreshToken]);

  const changeStep = useCallback((newStep: number | ((old: number) => number)) => {
    const resolved = typeof newStep === 'number' ? newStep : newStep(activeStep);
    const newUrl = new URL(window.location.href);
    newUrl.pathname = `/setup/${resolved}`;
    window.history.pushState(null, '', newUrl);
    // Force re-render via URL; the router param change will update activeStep.
    window.dispatchEvent(new PopStateEvent('popstate'));
  }, [activeStep]);

  return { steps, activeStep, changeStep };
}
