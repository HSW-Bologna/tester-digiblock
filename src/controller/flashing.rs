use tokio::process::Command;

pub async fn load_test_firmware() -> Option<i32> {
    println!("Loading test firmware");
    return Some(0);
    Command::new("st-flash")
    .args(&["write", "tmp/digiblock-hwtest.bin", "0x8000000"])
    .status()
    .await
    .ok()
    .and_then(|res| res.code())
}


pub async fn load_production_firmware() -> Option<i32> {
    println!("Loading production firmware");
    return Some(0);
    Command::new("st-flash")
    .args(&["write", "tmp/digiblock-hwtest.bin", "0x8000000"])
    .status()
    .await
    .ok()
    .and_then(|res| res.code())
}
