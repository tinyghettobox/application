import {Controller, useFormContext} from "react-hook-form";
import {FormControl, Grid, InputLabel, OutlinedInput, Typography} from "@mui/material";

export default function DevCredentials() {
  const {control} = useFormContext();

  return (
    <div>
      <Typography variant="h5">Dev app credentials</Typography>
      <br/>
      <Typography variant="h6">How does it work?</Typography>
      <p>
        The integration of Spotify works by letting spotifyd, an interface less spotify client, run on the system. It
        will be controlled by MuPiBox-rs through the Spotify API. Imagine this like MuPiBox-rs is your phone which plays
        music on a speaker, but the speaker is the same system. To enable MuPiBox-rs to control Spotify you need to
        configure relevant information to authorize on Spotify&apos;s API through the Authorization Code flow.
      </p>
      <Typography variant="h6">What you need to do now!</Typography>
      <p>
        The information MuPiBox-rs needs are the client id and secret key. In order to get them, you have to create a
        developer application at <a href={'https://developer.spotify.com'}>https://developer.spotify.com</a>. Use the
        following config to create it (only website and redirect uri are relevant):
      </p>
      <ul>
        <li><strong>App name</strong>: MuPiBox-rs</li>
        <li><strong>App description</strong>: Developer app for my mupibox-rs</li>
        <li><strong>Website</strong>: http://mupibox</li>
        <li><strong>Redirect URI</strong>: http://mupibox/api/spotify/callback</li>
      </ul>
      <p>
        After that you have to head back to the settings of the newly created app, copy the Client ID and client secret
        and fill them below.
      </p>

      <Grid container spacing={2} sx={{mt: '24px'}}>
        <Grid item xs={6}>
          <Controller
            name={'clientId'}
            rules={{required: true}}
            control={control}
            render={({field, fieldState}) =>
              <FormControl fullWidth error={fieldState.invalid}>
                <InputLabel id={"client-id-label"}>Client ID</InputLabel>
                <OutlinedInput label={"Client ID"} {...field} />
              </FormControl>
            }
          />
        </Grid>
        <Grid item xs={6}>
          <Controller
            name={'secretKey'}
            rules={{required: true}}
            control={control}
            render={({field, fieldState}) =>
              <FormControl fullWidth error={fieldState.invalid}>
                <InputLabel id={"secret-key-label"}>Secret Key</InputLabel>
                <OutlinedInput label={"Secret Key"} {...field} />
              </FormControl>
            }
          />
        </Grid>
      </Grid>
    </div>
  )
}