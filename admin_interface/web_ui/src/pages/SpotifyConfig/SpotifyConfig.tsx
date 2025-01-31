import {FormProvider, useForm} from "react-hook-form";
import SpotifyForm from "@/pages/SpotifyConfig/SpotifyForm";

export default function SpotifyConfig() {
  const {...formFns} = useForm({
    mode: 'onTouched',
    values: {
      clientId: '',
      secretKey: '',
      accessToken: '',
      refreshToken: '',
      expiresAt: '',
      username: '',
      password: ''
    }
  });

  return (
    <FormProvider {...formFns}>
      <main>
        <SpotifyForm />
      </main>
    </FormProvider>
  )
}