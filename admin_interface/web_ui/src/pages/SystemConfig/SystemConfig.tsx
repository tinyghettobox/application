import {Button, Grid, Typography} from "@mui/material";
import {useEffect} from "react";
import {useForm} from "react-hook-form";
import TimerConfig from "./TimerConfig";
import SystemSettings from "./SystemSettings";
import DisplaySettings from "./DisplaySettings";
import AudioSettings from "./AudioSettings";
import PowerSettings from "./PowerSettings";
import {notify} from "@/components/Notification";
import {getSystemConfig, putSystemConfig} from "@/util/api";

export default function SystemConfig() {
  const {formState, control, handleSubmit, reset} = useForm({
    mode: 'onTouched',
    values: {
      // timer
      sleepTimer: 60, // self handled
      idleShutdownTimer: 15, // self handled
      displayOffTimer: 5, // self handled
      // system
      hostname: 'mupibox', // command x
      cpuGovernor: 'performance', // command x
      overclockSdCard: false, // command
      logToRam: true, // command x
      waitForNetwork: true, // command x
      initialTurbo: false, // command x
      disableWarnings: false, // self handled
      swapEnabled: true, // command x
      // display
      hdmiRotate: 0, // command x
      lcdRotate: 0, // command x
      displayBrightness: 100, // command x
      displayResolutionX: 800, // self handled
      displayResolutionY: 400, // self handled
      // audio
      audioDevice: 'hifiberry-dac', // command x
      volume: 40, // command x
      powerOnVolume: 40, // ???
      maxVolume: 100, // self handled
      // power
      ledOnOffShimPin: 25, // command x
      ledBrightness: 100, // script mupi_start_led.sh
      ledBrightnessDimmed: 10, // script mupi_start_led.sh
      powerOffBtnDelay: 2, // script handled
      powerPin: 4,
      triggerPin: 17,
    }
  });

  useEffect(() => {
    (async () => {
      try {
        const config = await getSystemConfig();
        reset(config);
      } catch (e) {
        notify('error', `Could not load config: ${e}`);
      }
    })();
  }, [reset])

  const saveData = async (data: any) => {
    const notificationKey = Math.random().toString();
    notify('info', 'Saving...', undefined, notificationKey);

    try {
      const config = await putSystemConfig(data);
      reset(config);
      notify('success', `Saved :)`, 2000, notificationKey);
    } catch (e) {
      notify('error', `Saving failed: ${e}`, undefined, notificationKey);
    }
  }

  return (
    <main>
      <form onSubmit={handleSubmit(saveData)}>
        <Grid container alignItems={"center"}>
          <Grid item xs={10}>
            <Typography variant={'h4'} sx={{mb: '16px', mt: '32px'}}>
              Configuration
            </Typography>
          </Grid>
          <Grid item xs={2} textAlign={"right"}>
            <Button
              variant="contained"
              type={'submit'}
              disabled={!formState.isDirty || !formState.isValid || formState.isSubmitting}
            >
              Save
            </Button>
          </Grid>
        </Grid>
        <Typography variant="subtitle1" sx={{mb: 5}}>
          Here you can configure general settings of the MuPiBox-rs
        </Typography>
        <div style={{position: 'relative'}}>
          <TimerConfig control={control}/>
          <SystemSettings control={control}/>
          <DisplaySettings control={control}/>
          <AudioSettings control={control}/>
          <PowerSettings control={control}/>
        </div>
      </form>
    </main>
  )
}