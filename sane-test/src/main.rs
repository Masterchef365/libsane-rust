use libsane::*;

fn main() -> Result<()> {
    let sane = LibSane::init(None)?;

    let devices = sane.device_names(true)?;

    for device_desc in devices.iter() {
        println!("--- {} ---", device_desc.name.to_str().unwrap());
        let device = device_desc.open()?;
        for (idx, option) in device.options().enumerate() {
            println!("{}: {:#?}", idx, option);
        }
    }

    //let device = sane.open("plustek:libusb:001:006")?;

    Ok(())
}
