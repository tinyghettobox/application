import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  Checkbox,
  FormControl,
  FormControlLabel,
  FormHelperText,
  InputLabel,
  MenuItem,
  OutlinedInput,
  Select,
  Stack,
  Typography
} from "@mui/material";
import {Control, Controller} from "react-hook-form";

interface Props {
  control: Control<any>
}

export default function SystemSettings({control}: Props) {
  return (
    <Accordion>
      <AccordionSummary>
        <Typography variant={"h5"}>System settings</Typography>
      </AccordionSummary>
      <AccordionDetails>
        <Stack rowGap={3}>
          <div>
            <Controller
              name={'hostname'}
              rules={{required: true}}
              control={control}
              render={({field, fieldState}) =>
                <FormControl error={fieldState.invalid}>
                  <InputLabel id={"hostname-label"}>Hostname</InputLabel>
                  <OutlinedInput label={"Hostname"} {...field} />
                  <FormHelperText sx={{ml: 0}}>
                    The TinyGhettoBox will be visible in your LAN with this name
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'cpuGovernor'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl error={fieldState.invalid}>
                  <InputLabel id={"cpu-governor-label"}>CPU Governor</InputLabel>
                  <Select variant="outlined" labelId="cpu-governor-label" label={"CPU Governor"} {...field}>
                    <MenuItem value={"conservative"}>conservative</MenuItem>
                    <MenuItem value={"ondemand"}>ondemand</MenuItem>
                    <MenuItem value={"userspace"}>userspace</MenuItem>
                    <MenuItem value={"powersave"}>powersave</MenuItem>
                    <MenuItem value={"performance"}>performance</MenuItem>
                    <MenuItem value={"schedutil"}>schedutil</MenuItem>
                  </Select>
                  <FormHelperText sx={{ml: 0}}>
                    Try powersave (Limits CPU frequency to 600 MHz - Helps to avoid throtteling).
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'overclockSdCard'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl variant="standard" component={"fieldset"} error={fieldState.invalid}>
                  <FormControlLabel
                    control={<Checkbox {...field} checked={field.value}/>}
                    label={'Overclock SD card'}
                  />
                  <FormHelperText>
                    By default Raspberry Pi uses 17.5MB/s for SDCard IO. You can double the speed to 35MB/S. Be aware
                    you need at least a UHS-1 or faster SDCard and you can damage data or the SDCard itself!
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'logToRam'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl variant="standard" component={"fieldset"} error={fieldState.invalid}>
                  <FormControlLabel
                    control={<Checkbox {...field} checked={field.value}/>}
                    label={'Log to RAM'}
                  />
                  <FormHelperText>
                    To protect the microSD, the logs can be swapped out to RAM. However, logs can no longer be evaluated
                    after a restart.
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'waitForNetwork'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl variant="standard" component={"fieldset"} error={fieldState.invalid}>
                  <FormControlLabel
                    control={<Checkbox {...field} checked={field.value}/>}
                    label={'Wait for Network on boot'}
                  />
                  <FormHelperText>
                    Speeds up the boot time, but sometimes the boot process is to fast and you have to wait for the
                    network to be ready... Try it, if disabling this option works for you!
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'initialTurbo'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl variant="standard" component={"fieldset"} error={fieldState.invalid}>
                  <FormControlLabel
                    control={<Checkbox {...field} checked={field.value}/>}
                    label={'Initial Turbo'}
                  />
                  <FormHelperText>
                    Initial Turbo avoids throttling sometimes...
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'disableWarnings'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl variant="standard" component={"fieldset"} error={fieldState.invalid}>
                  <FormControlLabel
                    control={<Checkbox {...field} checked={field.value}/>}
                    label={'Disable Warnings (Throttling Warning)'}
                  />
                  <FormHelperText>
                    Enables or disables the lightning icon (warning)! In worst case, this option can cause you loose all
                    your data.
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'swapEnabled'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl variant="standard" component={"fieldset"} error={fieldState.invalid}>
                  <FormControlLabel
                    control={<Checkbox {...field} checked={field.value}/>}
                    label={'SWAP'}
                  />
                  <FormHelperText>
                    Enables or disables SWAP!
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
        </Stack>
      </AccordionDetails>
    </Accordion>
  )
}