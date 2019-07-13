use fprint_rs::{FPrint, FPrintError};

fn main() -> Result<(), FPrintError> {
    let fprint = FPrint::new()?;
    let discovered = fprint.discover();
    let device = discovered.get(0).expect("Device not found").open();

    if !device.supports_imaging() {
        eprintln!("This device does not have imaging capabilities.");
        return Ok(());
    }

    println!("Opened device. It's now time to scan your finger.");
    let image = device.capture_image(true)?;
    image.save_to_file("finger.pgm")?;
    image.standardize();
    image.save_to_file("finger_standardized.pgm")?;

    Ok(())
}
