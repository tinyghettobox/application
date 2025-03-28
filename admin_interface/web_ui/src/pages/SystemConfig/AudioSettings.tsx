import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  FormControl,
  FormHelperText,
  FormLabel,
  Grid,
  InputLabel,
  MenuItem,
  Select,
  Slider,
  Stack,
  TextField,
  Typography
} from "@mui/material";
import {Control, Controller} from "react-hook-form";

interface Props {
  control: Control<any>,
}

export default function AudioSettings({control}: Props) {

  return (
    <Accordion>
      <AccordionSummary>
        <Typography variant={"h5"}>Audio settings</Typography>
      </AccordionSummary>
      <AccordionDetails>
        <Stack rowGap={3}>
          <div>
            <Controller
              name={'audioDevice'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl error={fieldState.invalid}>
                  <InputLabel id={"audio-device-label"}>Audio device / Soundcard</InputLabel>
                  <Select
                    variant="outlined"
                    labelId={"audio-device-label"}
                    label={"Audio device / Soundcard"} {...field}>
                    <MenuItem value={"rpi-bcm2835-3.5mm"}>Onboard 3.5mm output</MenuItem>
                    <MenuItem value={"rpi-bcm2835-hdmi"}>Onboard HDMI output</MenuItem>
                    <MenuItem value={"hifiberry-amp"}>HiFiBerry AMP / AMP+</MenuItem>
                    <MenuItem value={"hifiberry-dac"}>HiFiBerry DAC / MiniAmp</MenuItem>
                    <MenuItem value={"hifiberry-dacplus"}>HiFiBerry DAC+ / DAC+ Pro / AMP2</MenuItem>
                    <MenuItem value={"usb-dac"}>Any USB Audio DAC (Auto detection)</MenuItem>
                  </Select>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'volume'}
              rules={{
                required: true, max: 100, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              control={control}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>Volume</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="Volume"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={100}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>Specify audio volume in percent</FormHelperText>
                </FormControl>
              }
            />
          </div>
          {/*<div>*/}
          {/*  <Controller*/}
          {/*    name={'powerOnVolume'}*/}
          {/*    rules={{*/}
          {/*      required: true, max: 100, min: 0, validate: (value) => !isNaN(Number(value))*/}
          {/*    }}*/}
          {/*    control={control}*/}
          {/*    render={({field, fieldState}) =>*/}
          {/*      <FormControl fullWidth error={fieldState.invalid}>*/}
          {/*        <FormLabel>Volume after power on</FormLabel>*/}
          {/*        <Grid container spacing={2} alignItems={"center"}>*/}
          {/*          <Grid item xs={1}>*/}
          {/*            <TextField*/}
          {/*              error={fieldState.invalid}*/}
          {/*              {...field}*/}
          {/*            />*/}
          {/*          </Grid>*/}
          {/*          <Grid item xs={5}>*/}
          {/*            <Slider*/}
          {/*              aria-label="Volume after power on"*/}
          {/*              valueLabelDisplay="auto"*/}
          {/*              step={1}*/}
          {/*              min={0}*/}
          {/*              max={100}*/}
          {/*              {...field}*/}
          {/*            />*/}
          {/*          </Grid>*/}
          {/*        </Grid>*/}
          {/*        <FormHelperText sx={{ml: 0}}>Specify audio volume in percent</FormHelperText>*/}
          {/*      </FormControl>*/}
          {/*    }*/}
          {/*  />*/}
          {/*</div>*/}
          <div>
            <Controller
              name={'maxVolume'}
              rules={{
                required: true, max: 100, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              control={control}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>Maximum volume</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="Maximum volume"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={100}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>Specify maximum audio volume in percent</FormHelperText>
                </FormControl>
              }
            />
          </div>
        </Stack>
      </AccordionDetails>
    </Accordion>
  )
}