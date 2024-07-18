import {FormEvent, useEffect, useState} from "react";
import {
  Box,
  Button,
  CircularProgress,
  Grid,
  Stack,
  Step,
  StepButton,
  StepLabel,
  Stepper,
  Typography
} from "@mui/material";
import {useFormContext} from "react-hook-form";
import {notify} from "@/components/Notification";
import {getSpotifyConfig, putSpotifyConfig} from "@/util/api";
import {useSteps} from "@/pages/SpotifyConfig/useSteps";
import DevCredentials from "@/pages/SpotifyConfig/steps/DevCredentials";
import Authorize from "@/pages/SpotifyConfig/steps/Authorize";
import AccountCredentials from "@/pages/SpotifyConfig/steps/AccountCredentials";

export default function SpotifyForm() {
  const form = useFormContext();
  const {steps, activeStep, changeStep} = useSteps();
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const config = await getSpotifyConfig();
        form.reset(config);

        changeStep(() => {
          if (config.refreshToken) {
            return 2;
          }
          if (config.clientId) {
            return 1;
          }
          return 0;
        });
      } catch (e) {
        notify('error', `Could not load config: ${e}`);
      } finally {
        setLoading(false);
      }
    })();
  }, [changeStep, form.reset]); // eslint-disable-line react-hooks/exhaustive-deps

  const saveData = async (data: any) => {
    const notificationKey = Math.random().toString();
    notify('info', 'Saving...', undefined, notificationKey);

    try {
      const config = await putSpotifyConfig(data);
      form.reset(config);
      notify('success', `Saved :)`, 2000, notificationKey);

      changeStep(() => {
        if (config.refreshToken) {
          return 2;
        }
        if (config.clientId) {
          return 1;
        }
        return 0;
      });
    } catch (e) {
      notify('error', `Saving failed: ${e}`, undefined, notificationKey);
    }
  }

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    // Just change step if the step is completed already and the firm did not change
    if (steps[activeStep].completed && !form.formState.isDirty) {
      changeStep(step => {
        if (step + 1 < steps.length) {
          return step + 1;
        }
        return step;
      });
      return;
    }

    const isValid = await form.trigger();
    if (isValid) {
      const data = form.getValues();
      await saveData(data);
    }
  }

  const handleStepClick = (step: number) => {
    return () => {
      changeStep(step);
    }
  }

  const handleBack = () => {
    changeStep(step => {
      if (step > 0) {
        return step - 1;
      }
      return 0;
    })
  }

  return (
    <form onSubmit={handleSubmit}>
      <Grid container alignItems={"center"}>
        <Grid item xs={10}>
          <Typography variant="h4" sx={{mb: '24px', mt: '48px'}}>
            Spotify
          </Typography>
        </Grid>
      </Grid>
      <Typography variant="subtitle1" sx={{mb: 5}}>
        Here you can connect your MuPiBox-rs to Spotify. Each step explains what you have to do.
      </Typography>
      {loading ? (
        <CircularProgress />
      ) : (
        <>
          <Stepper activeStep={activeStep}>
            {steps.map((step, stepNumber) =>
              <Step key={step.label} completed={step.completed}>
                {steps[stepNumber - 1]?.completed || steps[stepNumber].completed ?
                  <StepButton
                    onClick={handleStepClick(stepNumber)}
                    optional={step.optional}
                    disabled={false}
                  >
                    <span style={{cursor: 'pointer'}}>
                      {step.label}
                    </span>
                  </StepButton>
                  :
                  <StepLabel>{step.label}</StepLabel>
                }
              </Step>
            )}
          </Stepper>
          <Box sx={{p: '48px'}}>
            {activeStep === 0 && <DevCredentials/>}
            {activeStep === 1 && <Authorize/>}
            {activeStep === 2 && <AccountCredentials/>}
            <Stack direction={'row'} spacing={2} sx={{mt: '32px'}} justifyContent={'end'}>
              {activeStep !== 0 && (
                <Button onClick={handleBack}>
                  Back
                </Button>
              )}
              <Button
                variant="contained"
                type={'submit'}
                disabled={!steps[activeStep].completed &&
                  (!form.formState.isDirty || !form.formState.isValid || form.formState.isSubmitting)}
              >
                {activeStep === 2 ? 'Save' : 'Next'}
              </Button>
            </Stack>
          </Box>
        </>
      )}
    </form>
  );
}