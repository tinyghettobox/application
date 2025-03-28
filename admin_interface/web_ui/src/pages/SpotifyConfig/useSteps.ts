import {useFormContext} from "react-hook-form";
import {useCallback, useState} from "react";
import {useParams} from "react-router-dom";

export function useSteps() {
  const {step} = useParams();
  const [activeStep, setActiveStep] = useState(parseInt(step || '') || 0);
  const {watch} = useFormContext();

  const [clientId, secretKey, refreshToken, username, password] = watch([
    'clientId',
    'secretKey',
    'refreshToken',
    'username',
    'password'
  ]);

  const steps = [
    {
      label: 'Dev app credentials',
      completed: !!(clientId && secretKey)
    }, {
      label: 'Authorize',
      completed: !!refreshToken
    }, {
      label: 'Account credentials',
      completed: !!(username && password),
      optional: true
    }
  ];

  const changeStep = useCallback((newStep: number | ((oldStep: number) => number)) => {
    setActiveStep(oldStep => {
      const step = typeof newStep === 'number' ? newStep : newStep(oldStep);

      const newUrl = new URL(window.location.href);
      newUrl.pathname = `/spotifyConfig/${step}`;
      window.history.pushState(null, '', newUrl);

      return step;
    })
  }, [setActiveStep]);

  return {steps, activeStep, changeStep};
}