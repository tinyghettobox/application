import { FormProvider, useForm } from 'react-hook-form';
import SetupForm from './SetupForm';

export interface SetupFormValues {
  ssid: string;
  wifiPassword: string;
  security: 'wpa' | 'wpa2' | 'wpa3' | 'wep' | 'nopass';
  useSpotify: boolean | null;
  spotifyClientId: string;
  spotifySecretKey: string;
  spotifyRefreshToken: string;
}

export default function Setup() {
  const formFns = useForm<SetupFormValues>({
    mode: 'onTouched',
    defaultValues: {
      ssid: '',
      wifiPassword: '',
      security: 'wpa2',
      useSpotify: null,
      spotifyClientId: '',
      spotifySecretKey: '',
      spotifyRefreshToken: '',
    },
  });

  return (
    <FormProvider {...formFns}>
      <SetupForm />
    </FormProvider>
  );
}
