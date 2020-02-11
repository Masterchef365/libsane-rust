use libsane::*;

fn main() -> Result<()> {
    let sane = LibSane::init(None)?;

    for device_desc in sane.list_devices(true)? {
        println!("--- {} ---", device_desc.name.to_str().unwrap());
        let device = device_desc.open(&sane)?;
        for (idx, option) in device.options().enumerate() {
            println!("{}: {:#?}", idx, option);
        }
    }

    //let device = sane.open_device("plustek:libusb:001:006")?;

    Ok(())
}
