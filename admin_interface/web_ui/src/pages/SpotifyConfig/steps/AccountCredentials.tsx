import {Controller, useFormContext} from "react-hook-form";
import {FormControl, Grid, InputLabel, OutlinedInput, Typography} from "@mui/material";

export default function AccountCredentials() {
  const {control} = useFormContext();

  return (
    <div>
      <Typography variant="h5">Account credentials</Typography>
      <br/>
      <Typography variant="h6">Why do you need this?</Typography>
      <p>
        By default the spotifyd instance, which acts like a speaker, will be accessible to the whole network.
        Meaning all devices in the same network can start playback on the TinyGhettoBox. In case you want to prevent
        that,
        you can specify which account is allowed to use the spotifyd instance. The spotifyd instance will login to the
        account and register itself there instead of notifying the whole network about the availability. <br/><br/>
        <i>When you can&apos;t see your speaker your credentials may be wrong</i>
      </p>

      <Grid container spacing={2} sx={{mt: '24px'}}>
        <Grid item xs={6}>
          <Controller
            name={'username'}
            control={control}
            render={({field, fieldState}) =>
              <FormControl fullWidth error={fieldState.invalid}>
                <InputLabel id={"username-label"}>Username</InputLabel>
                <OutlinedInput label={"Username"} {...field} />
              </FormControl>
            }
          />
        </Grid>
        <Grid item xs={6}>
          <Controller
            name={'password'}
            control={control}
            render={({field, fieldState}) =>
              <FormControl fullWidth error={fieldState.invalid}>
                <InputLabel id={"password-label"}>Password</InputLabel>
                <OutlinedInput label={"Password"} type="password" {...field} />
              </FormControl>
            }
          />
        </Grid>
      </Grid>
    </div>
  )
}