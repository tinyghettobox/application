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

export default function PowerSettings({control}: Props) {

  return (
    <Accordion>
      <AccordionSummary>
        <Typography variant={"h5"}>Power settings</Typography>
      </AccordionSummary>
      <AccordionDetails>
        <Stack rowGap={3}>
          <div>
            <Controller
              name={'powerOffBtnDelay'}
              control={control}
              rules={{
                required: true, max: 5, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>Power off button delay</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="Power off button delay"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={5}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>
                    How many seconds the button need to be pressed to shutdown
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'ledPin'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl error={fieldState.invalid}>
                  <InputLabel id={"led-pin-label"}>LED GPIO OnOffShim</InputLabel>
                  <Select
                    variant="outlined"
                    labelId={"led-pin-label"}
                    label={"LED GPIO OnOffShim"} {...field}>
                    <MenuItem value={0}>None</MenuItem>
                    <MenuItem value={4}>GPIO 4</MenuItem>
                    <MenuItem value={17}>GPIO 17</MenuItem>
                    <MenuItem value={18}>GPIO 18</MenuItem>
                    <MenuItem value={21}>GPIO 21</MenuItem>
                    <MenuItem value={21}>GPIO 21</MenuItem>
                    <MenuItem value={22}>GPIO 22</MenuItem>
                    <MenuItem value={23}>GPIO 23</MenuItem>
                    <MenuItem value={24}>GPIO 24</MenuItem>
                    <MenuItem value={25}>GPIO 25</MenuItem>
                    <MenuItem value={27}>GPIO 27</MenuItem>
                  </Select>
                  <FormHelperText sx={{ml: 0}}>
                    Which GPIO pin to use for power button led control? If you use OnOffShim GPIOs 4 and 17
                    are used by it already. GPIOs 18 and 21 are used by HifiBerry MiniAmp. Just use free GPIOs to avoid
                    system errors.
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'ledBrightness'}
              rules={{
                required: true, max: 100, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              control={control}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>LED Brightness normal</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="LED Brightness normal"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={100}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>Brightness of power button led in percent</FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'ledBrightnessDimmed'}
              rules={{
                required: true, max: 100, min: 0, validate: (value) => !isNaN(Number(value))
              }}
              control={control}
              render={({field, fieldState}) =>
                <FormControl fullWidth error={fieldState.invalid}>
                  <FormLabel>LED Brightness dimmed</FormLabel>
                  <Grid container spacing={2} alignItems={"center"}>
                    <Grid item xs={1}>
                      <TextField
                        error={fieldState.invalid}
                        {...field}
                      />
                    </Grid>
                    <Grid item xs={5}>
                      <Slider
                        aria-label="LED Brightness dimmed"
                        valueLabelDisplay="auto"
                        step={1}
                        min={0}
                        max={100}
                        {...field}
                      />
                    </Grid>
                  </Grid>
                  <FormHelperText sx={{ml: 0}}>Brightness of dimmed power button led in percent</FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'powerOffPin'}
              control={control}
              render={({field, fieldState}) =>
                <FormControl error={fieldState.invalid}>
                  <InputLabel id={"power-off-pin-label"}>Power Off GPIO Pin (OnOffShim)</InputLabel>
                  <Select
                    variant="outlined"
                    labelId={"power-off-pin-label"}
                    label={"Power Off GPIO Pin (OnOffShim)"} {...field}>
                    <MenuItem value={4}>GPIO 4</MenuItem>
                    <MenuItem value={17}>GPIO 17</MenuItem>
                    <MenuItem value={18}>GPIO 18</MenuItem>
                    <MenuItem value={21}>GPIO 21</MenuItem>
                    <MenuItem value={21}>GPIO 21</MenuItem>
                    <MenuItem value={22}>GPIO 22</MenuItem>
                    <MenuItem value={23}>GPIO 23</MenuItem>
                    <MenuItem value={24}>GPIO 24</MenuItem>
                    <MenuItem value={25}>GPIO 25</MenuItem>
                    <MenuItem value={27}>GPIO 27</MenuItem>
                  </Select>
                  <FormHelperText sx={{ml: 0}}>
                    Which GPIO pin to use for telling OnOffShim to power off. Usually pin 4.
                  </FormHelperText>
                </FormControl>
              }
            />
          </div>
          <div>
            <Controller
              name={'cutPin'}
              control={control}
              render={({field, fieldState}) => {
                debugger;
                return 
                <FormControl error={fieldState.invalid}>
                  <InputLabel id={"cut-pin-label"}>Cut GPIO Pin (OnOffShim)</InputLabel>
                  <Select
                    variant="outlined"
                    labelId={"cut-pin-label"}
                    label={"Cut GPIO Pin (OnOffShim)"} {...field}>
                    <MenuItem value={4}>GPIO 4</MenuItem>
                    <MenuItem value={17}>GPIO 17</MenuItem>
                    <MenuItem value={18}>GPIO 18</MenuItem>
                    <MenuItem value={21}>GPIO 21</MenuItem>
                    <MenuItem value={21}>GPIO 21</MenuItem>
                    <MenuItem value={22}>GPIO 22</MenuItem>
                    <MenuItem value={23}>GPIO 23</MenuItem>
                    <MenuItem value={24}>GPIO 24</MenuItem>
                    <MenuItem value={25}>GPIO 25</MenuItem>
                    <MenuItem value={27}>GPIO 27</MenuItem>
                  </Select>
                  <FormHelperText sx={{ml: 0}}>
                    Which GPIO pin to use for telling OnOffShim to cut the power. This is different to power off pin as this is set low by the system and tells
                    on off shim to fully cut power.
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