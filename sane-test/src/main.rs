
/*
unsafe extern "C" fn auth_callback(
    resource: SANE_String_Const,
    username: *mut SANE_Char,
    password: *mut SANE_Char,
) {
}
*/

use libsane::*;

fn main() -> Result<(), SaneError> {
    let sane = LibSane::init()?;
    //let devices = sane.get_devices(true)?;
    /*
    for device in devices.iter() {
        println!("{}", device.name.to_str().unwrap());
    }
    */
    let device = sane.open("plustek:libusb:001:006")?;
    let options = OptionDescriptorIterator::new(&device);
    for option in options {
        println!("{:#?}", option);
    }
    /*
    let devices = SaneDeviceList::get_devices(&sane, true)?;
    for device in devices.iter() {
        let device = device.open();
        drop(device);
    }
    */

    /*
    let mut i = 0;
    loop {
    let ptr = devices.wrapping_offset(i);
    if (*ptr).is_null() {
    break;
    }
    i += 1;
    let device = *ptr;
    let name = CStr::from_ptr((*device).name);
    let vendor = CStr::from_ptr((*device).vendor);
    let model = CStr::from_ptr((*device).model);
    let type_ = CStr::from_ptr((*device).type_);
    println!("{:?} {:?} {:?} {:?}", name, vendor, model, type_);
    }
    std::alloc::dealloc(devices);
    */
    Ok(())
}
