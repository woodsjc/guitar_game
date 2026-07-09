use cpal::traits::{DeviceTrait, HostTrait};

pub fn get_scarlet(host: &cpal::Host) -> Option<cpal::Device> {
    host.input_devices()
        .ok()?
        .find(|d| d.to_string().contains("Scarlett"))
}

pub fn print_all_devices(host: &cpal::Host) {
    if let Ok(devices) = host.input_devices() {
        println!("Devices:");
        for device in devices {
            let desc = device.description();
            let config = device.default_input_config();
            println!("\t{}", device.to_string());
            if let Ok(d) = desc {
                if let Some(mfr) = d.manufacturer() {
                    println!("\t  manufacturer: {}", mfr);
                }
                println!("\t  direction: {:?}", d.direction());
            }
            if let Ok(c) = config {
                println!(
                    "\t  config: {} Hz, {} ch, {:?}",
                    c.sample_rate(),
                    c.channels(),
                    c.sample_format(),
                );
            }
        }
    }
}
