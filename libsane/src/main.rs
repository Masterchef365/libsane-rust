use libsane::*;

fn main() -> Result<()> {
    let sane = LibSane::init(None)?;
    let device = sane.open_device("plustek:libusb:001:006")?;
    //let options = device.options().collect::<Vec<_>>();
    //let m = std::ffi::CString::new("resolution").unwrap();
    //let res_option = options.iter().find(|&x| x.name == Some(m.as_c_str())).unwrap();
    for option in device.options() {
        println!("{:?}", option);
        let value = device.get_option(&option)?;
        if let Some(v) = &value {
            device.set_option(&option, v)?;
        }
        let v2 = device.get_option(&option)?;
        assert_eq!(value, v2);
        println!("\t{:?}", value);
    }
    //println!("{:#?}", res_option);
    //println!("{:#?}", device.get_option(&res_option));
    //device.set_option(&res_option, &mut Value::Int(1200))?;
    //println!("{:#?}", device.get_option(&res_option));
    //drop(device);
    //drop(sane);

    Ok(())
}
