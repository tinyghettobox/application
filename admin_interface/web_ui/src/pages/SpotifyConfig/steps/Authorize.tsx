import {Box, Button, Typography} from "@mui/material";
import {useFormContext} from "react-hook-form";
import {Check} from "@mui/icons-material";

export default function Authorize() {
  const {watch} = useFormContext();

  const refreshToken = watch('refreshToken');
  const expiresAt = watch('expiresAt');
  const tokenIsValid = expiresAt && new Date(expiresAt).getTime() > Date.now() || false;

  const handleAuth = () => {
    // window.open('/api/SpotifyAddForm/auth', '_blank');
    window.location.href = '/api/spotify/auth';
  }

  return (
    <div>
      <Typography variant="h5">Authorize</Typography>
      <p>
        By clicking the button below, you will be redirected to the authorization page of Spotify. You have to login
        with the Spotify account you want to use on the TinyGhettoBox, and authorize the developer application you
        created
        in the previous step to control your spotify account and devices. Indeed this can be the same account but this
        authorization method is required by Spotify&apos;s rules when accessing sensitive data, which device and
        playback is
        considered to be.
      </p>

      <Box sx={{width: '100%', textAlign: 'center', p: '32px'}}>
        <Button onClick={handleAuth} disabled={tokenIsValid} variant="contained">
          {!!refreshToken ? <><Check/> Authorized</> : 'Authorize'}
        </Button>
      </Box>
    </div>
  )
}
