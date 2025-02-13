import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  FormControl, FormHelperText, FormLabel,
  Grid, InputLabel, MenuItem, Select,
  Slider, Stack,
  TextField,
  Typography
} from "@mui/material";
import {Control, Controller} from "react-hook-form";

interface Props {
  control: Control<any>
}

export default function DisplaySettings({control}: Props) {
  return (
    <Accordion>
      <AccordionSummary>
        <Typography variant={"h5"}>Display settings</Typography>
      </AccordionSummary>
      <AccordionDetails>
        <Stack rowGap={3}>
          <div>
            <Controller
              name={'displayBrightness'}
              rules={{
                required: true, max: 100, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              control={control}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>Display brightness</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        // size={'small'}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="Display brightness"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={100}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>Specify brightness in percent</FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'hdmiRotate'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl error={fieldState.invalid}>
                  <InputLabel id={"hdmi-rotate-label"}>HDMI Rotate</InputLabel>
                  <Select variant="outlined" labelId="hdmi-rotate-label" label={"HDMI Rotate"} {...field}>
                    <MenuItem value={0}>No rotation</MenuItem>
                    <MenuItem value={1}>90 degrees</MenuItem>
                    <MenuItem value={2}>180 degrees</MenuItem>
                    <MenuItem value={3}>270 degrees</MenuItem>
                    <MenuItem value={0x10000}>Horizontal flip</MenuItem>
                    <MenuItem value={0x20000}>Vertical flip</MenuItem>
                  </Select>
                  <FormHelperText sx={{ml: 0}}>
                    Rotate HDMI graphic output. Configures the display_hdmi_rotate property.
                    <a
                      target={"_blank"}
                      href={"https://www.raspberrypi.com/documentation/computers/configuration.html#legacy-graphics-driver"}
                    >
                      Read more
                    </a>
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'lcdRotate'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl error={fieldState.invalid}>
                  <InputLabel id={"lcd-rotate-label"}>LCD Rotate</InputLabel>
                  <Select variant="outlined" labelId="lcd-rotate-label" label={"LCD Rotate"} {...field}>
                    <MenuItem value={0}>No rotation</MenuItem>
                    <MenuItem value={1}>90 degrees</MenuItem>
                    <MenuItem value={2}>180 degrees</MenuItem>
                    <MenuItem value={3}>270 degrees</MenuItem>
                    <MenuItem value={0x10000}>Horizontal flip</MenuItem>
                    <MenuItem value={0x20000}>Vertical flip</MenuItem>
                  </Select>
                  <FormHelperText sx={{ml: 0}}>
                    Rotate attaches LCD panel graphic output. Configures the display_lcd_rotate property.
                    <a
                      target={"_blank"}
                      href={"https://www.raspberrypi.com/documentation/computers/configuration.html#legacy-graphics-driver"}
                    >
                      Read more
                    </a>
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <FormControl>
              <FormLabel sx={{mb: 1}}>Display resolution</FormLabel>
              <Grid container columnGap={2}>
                <Grid item xs={3}>
                  <Controller
                    name={"displayResolutionX"}
                    control={control}
                    render={({field, fieldState}) =>
                      <TextField
                        variant="outlined"
                        label={"X"}
                        error={fieldState.invalid}
                        {...field}
                        onChange={event => {
                          field.onChange({...event, target: {value: parseInt(event.target.value)}}); // @ts-ignore
                        }}
                      />
                    }
                  />
                </Grid>
                <Grid item alignSelf={"center"}>
                  x
                </Grid>
                <Grid item xs={3}>
                  <Controller
                    name={"displayResolutionY"}
                    control={control}
                    render={({field, fieldState}) =>
                      <TextField
                        variant="outlined"
                        label={"Y"}
                        error={fieldState.invalid}
                        {...field}
                        onChange={event => {
                          field.onChange({...event, target: {value: parseInt(event.target.value)}}); // @ts-ignore
                        }}
                      />
                    }
                  />
                </Grid>
              </Grid>

              <FormHelperText sx={{ml: 0}}>
                Display resolution in pixels
              </FormHelperText>
            </FormControl>
          </div>
        </Stack>
      </AccordionDetails>
    </Accordion>
  )
}