mod common;

use failure::Error;
use fprint_rs::{FPrint, IdentifyResult};
use rusqlite::NO_PARAMS;
use std::collections::HashMap;

fn main() -> Result<(), Error> {
    let fprint = FPrint::new()?;
    let discovered = fprint.discover();
    let device = discovered.get(0).expect("Device not found").open();
    let (fingers, users) = load_fingers()?;

    loop {
        let identity_result = device.identify_finger_image(&fingers);
        if identity_result.is_err() {
            eprintln!("Error: {:?}", identity_result);
            continue;
        }

        match identity_result.unwrap() {
            IdentifyResult::Matched(offset) => match users.get(&offset) {
                Some(user_id) => println!("Found finger for user with id {}", user_id),
                None => eprintln!("Unknown offset"),
            },
            IdentifyResult::Error(e) => eprintln!("Identity error: {}", e),
        }
    }
}

fn load_fingers() -> Result<(Vec<Vec<u8>>, HashMap<usize, i32>), Error> {
    let fingers = vec![];
    let user_offsets = HashMap::new();
    let result = rusqlite::Connection::open(crate::common::DB_PATH)?
        .prepare("SELECT DISTINCT * FROM fingers")?
        .query_map(NO_PARAMS, |row| {
            let id: i32 = row.get(1)?;
            let length: isize = row.get(3)?;
            let finger: Vec<u8> = row.get(2)?;
            assert_eq!(finger.len(), length as usize);

            Ok((id, finger))
        })?
        .filter_map(|item| item.ok())
        .enumerate()
        .fold(
            (fingers, user_offsets),
            |(mut fingers, mut user_offsets), (offset, (user_id, finger))| {
                fingers.push(finger);
                user_offsets.insert(offset, user_id);

                (fingers, user_offsets)
            },
        );

    Ok(result)
}
