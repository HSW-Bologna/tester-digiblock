use std::{fs, path::PathBuf};
use tokio::process::Command;

use crate::controller::worker;

pub async fn load_test_firmware() -> Option<i32> {
    println!("Loading test firmware");

    worker::reset().await;

    Command::new("openocd")
        .args(&["-f", "./binaries/openocd-test.cfg"])
        .status()
        .await
        .ok()
        .and_then(|res| res.code())
}

pub async fn load_production_firmware() -> Option<i32> {
    let binary = get_production_firmware_path()
        .into_os_string()
        .into_string()
        .unwrap();
    println!("Loading production firmware {}", binary);

    worker::reset().await;

    Command::new("openocd")
        .args(&["-f", "./binaries/openocd-production.cfg"])
        .status()
        .await
        .ok()
        .and_then(|res| res.code())
    //Command::new("st-flash") .args(&["--reset", "write", binary.as_str(), "0x8000000"]) .status() .await .ok() .and_then(|res| res.code())
}

pub fn get_production_firmware_version() -> String {
    get_production_firmware_path()
        .file_name()
        .and_then(|v| {
            v.to_str()
                .map(|v| v.replace("digiblock-production-", "").replace(".hex", ""))
        })
        .unwrap_or("".into())
}

fn get_production_firmware_path() -> PathBuf {
    if let Ok(paths) = fs::read_dir("binaries") {
        for path in paths {
            let name = path.as_ref().unwrap().file_name().into_string().unwrap();
            if name.starts_with("digiblock-production-") && name.ends_with(".hex") {
                return path.unwrap().path();
            }
        }
        "".into()
    } else {
        "".into()
    }
}
