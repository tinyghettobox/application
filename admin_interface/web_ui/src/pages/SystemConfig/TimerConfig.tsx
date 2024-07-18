import {
  Accordion,
  AccordionDetails,
  AccordionSummary, FormControl, FormHelperText,
  FormLabel,
  Grid,
  Slider, Stack,
  TextField,
  Typography
} from "@mui/material";
import {Control, Controller} from "react-hook-form";

interface Props {
  control: Control<any>
}

export default function TimerConfig({control}: Props) {
  return (
    <Accordion>
      <AccordionSummary>
        <Typography variant={"h5"}>Timer settings</Typography>
      </AccordionSummary>
      <AccordionDetails>
        <Stack rowGap={3}>
          <div>
            <Controller
              name={'sleepTimer'}
              rules={{
                required: true, max: 360, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              control={control}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>Sleep timer</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="Sleeptimer"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={360}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>How many minutes to shut down MuPiBox</FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'idleShutdownTimer'}
              rules={{
                required: true, max: 60, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              control={control}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>Idle shutdown timer</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="Idle shutdown timer"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={60}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>
                    How many minutes can the Mupibox be idle (without playing) before it gets shutdown
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'displayOffTimer'}
              rules={{
                required: true, max: 60, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              control={control}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>Display turn off timer</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="Display turn off timer"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={60}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>
                    How many minutes after boot the display should be turned off? <br />
                    <i>Note: Depending on the display, the screen goes black but the backlight remains on.</i>
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