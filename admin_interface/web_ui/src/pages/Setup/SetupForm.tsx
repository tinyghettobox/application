import { Box, Step, StepButton, StepLabel, Stepper, Typography } from '@mui/material';
import { useSetupSteps, STEP_WIFI, STEP_SPOTIFY_QUESTION, STEP_SPOTIFY_CREDENTIALS, STEP_SPOTIFY_AUTHORIZE, STEP_COMPLETE } from './useSetupSteps';
import WifiSetup from './steps/WifiSetup';
import SpotifyQuestion from './steps/SpotifyQuestion';
import SpotifyCredentials from './steps/SpotifyCredentials';
import SpotifyAuthorize from './steps/SpotifyAuthorize';
import Complete from './steps/Complete';

export default function SetupForm() {
  const { steps, activeStep, changeStep } = useSetupSteps();

  const handleStepClick = (stepNumber: number) => () => {
    if (steps[stepNumber - 1]?.completed || steps[stepNumber].completed) {
      changeStep(stepNumber);
    }
  };

  return (
    <Box sx={{ p: '48px' }}>
      <Typography variant="h4" sx={{ mb: 3, mt: 2 }}>
        Welcome to tinyghettobox
      </Typography>
      <Typography variant="subtitle1" sx={{ mb: 5 }}>
        Let's get you set up. Each step will guide you through the configuration.
      </Typography>

      <Stepper activeStep={activeStep}>
        {steps.map((step, stepNumber) => (
          <Step key={step.label} completed={step.completed}>
            {steps[stepNumber - 1]?.completed || steps[stepNumber].completed ? (
              <StepButton onClick={handleStepClick(stepNumber)}>
                {step.label}
              </StepButton>
            ) : (
              <StepLabel optional={step.optional ? <span>Optional</span> : undefined}>
                {step.label}
              </StepLabel>
            )}
          </Step>
        ))}
      </Stepper>

      <Box sx={{ pt: '48px' }}>
        {activeStep === STEP_WIFI && <WifiSetup onComplete={() => changeStep(STEP_SPOTIFY_QUESTION)} />}
        {activeStep === STEP_SPOTIFY_QUESTION && (
          <SpotifyQuestion
            onYes={() => changeStep(STEP_SPOTIFY_CREDENTIALS)}
            onNo={() => changeStep(STEP_COMPLETE)}
          />
        )}
        {activeStep === STEP_SPOTIFY_CREDENTIALS && (
          <SpotifyCredentials onComplete={() => changeStep(STEP_SPOTIFY_AUTHORIZE)} onBack={() => changeStep(STEP_SPOTIFY_QUESTION)} />
        )}
        {activeStep === STEP_SPOTIFY_AUTHORIZE && (
          <SpotifyAuthorize onComplete={() => changeStep(STEP_COMPLETE)} onBack={() => changeStep(STEP_SPOTIFY_CREDENTIALS)} />
        )}
        {activeStep === STEP_COMPLETE && <Complete />}
      </Box>
    </Box>
  );
}
