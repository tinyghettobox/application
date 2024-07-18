use std::process::Command;
use tracing::{debug};
use crate::error::Problem;

macro_rules! run {
    ($cmd: expr $(, $arg: tt)*) => {
        if cfg!(target_os = "linux") {
            let cmd = format!($cmd, $(,$arg)*);
            debug!("Executing command on non-linux system: {}", cmd);
            let result = Command::new("sh").arg("-c").arg(cmd).output().expect("failed to execute process");
            if result.status.success() {
                return Ok(())
            } else {
                return Err(Problem::internal_error(String::from_utf8(result.stderr).unwrap(), None))
            }
        } else {
            let cmd = format!($cmd, $(,$arg)*);
            debug!("Not executing command on non-linux system: {}", cmd);
            Ok(())
        }
    };
}

pub fn set_spotifyd_config(field: &str, value: &str) -> Result<(), Problem> {
    run!("sudo sed -i 's/{field}\\s*=.*/{field} = \"{value}\"/' /etc/spotifyd/spotifyd.conf")?;
    run!("sudo systemctl restart spotifyd.service")
}

pub fn set_hostname(hostname: String) -> Result<(), Problem> {
    run!("sudo /boot/dietpi/func/change_hostname '{hostname}'")
}

pub fn set_cpu_governor(governor: String) -> Result<(), Problem> {
    run!("sudo -s sed -i 's/CONFIG_CPU_GOVERNOR=.*/CONFIG_CPU_GOVERNOR={governor}/' /boot/dietpi.txt")?;
    run!("sudo -s /boot/dietpi/func/dietpi-set_cpu")
}

pub fn set_log_to_ram(log_to_ram: bool) -> Result<(), Problem> {
    if log_to_ram {
        run!("sudo sed '/\\/home\\/dietpi\\/.pm2\\/logs/d' /etc/fstab > /tmp/fstab")
    } else {
        run!("sudo sed -i '/^tmpfs \\/var\\/log.*/a tmpfs \\/home\\/dietpi\\/.pm2\\/logs tmpfs size=50M,noatime,lazytime,nodev,nosuid,mode=1777' /etc/fstab")
    }
}

pub fn set_wait_for_network(wait_for_network: bool) -> Result<(), Problem> {
    let flag = if wait_for_network { 1 } else { 0 };
    run!("sudo /boot/dietpi/func/dietpi-set_software boot_wait_for_network {flag}")
}

pub fn set_swap_enabled(swap_enabled: bool) -> Result<(), Problem> {
    let flag = if swap_enabled { 1 } else { 0 };
    run!("sudo /boot/dietpi/func/dietpi-set_swapfile {flag}")
}

pub fn set_initial_turbo(initial_turbo: bool) -> Result<(), Problem> {
    let turbo = if initial_turbo { 30 } else { 0 };
    run!("sudo sed -i 's/initial_turbo=.*/initial_turbo={turbo}/' /boot/config.txt")
}

pub fn set_hdmi_rotate(hdmi_rotate: i32) -> Result<(), Problem> {
    run!("sudo sed -i 's/display_hdmi_rotate=.*/display_hdmi_rotate={hdmi_rotate}/' /boot/config.txt")
}

pub fn set_lcd_rotate(lcd_rotate: i32) -> Result<(), Problem> {
    run!("sudo sed -i 's/display_lcd_rotate=.*/display_lcd_rotate={lcd_rotate}/' /boot/config.txt")
}

pub fn set_display_brightness(brightness: i32) -> Result<(), Problem> {
    run!("sudo echo '{brightness}' > /sys/class/backlight/rpi_backlight/brightness")
}

pub fn set_audio_device(device: String) -> Result<(), Problem> {
    run!("sudo /boot/dietpi/func/dietpi-set_hardware soundcard '{device}'")
}

pub fn _set_volume(volume: i32) -> Result<(), Problem> {
    run!("sudo /usr/bin/amixer sset Master '{volume}%'")
}

pub fn set_overclock_sd_card(overclock: bool) -> Result<(), Problem> {
    let result = run!("sudo sed -i  's/\\ndtparam=sd_overclock=100//' /boot/config.txt");
    if overclock {
        result?;
        run!("sudo echo 'dtparam=sd_overclock=100' >> /boot/config.txt")
    } else {
        result
    }
}

pub fn set_led_on_off_shim_pin(pin: i32) -> Result<(), Problem> {
    run!("sudo sed -i 's/--gpio \\d+/--gpio {pin}/' /etc/init.d/pi-blaster.boot.sh")
}
