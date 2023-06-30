//use iced_futures::futures::TryFutureExt;
//use tokio::process::Command;

pub async fn load_test_firmware() -> Option<i32> {
    return Some(0);
    /*Command::new("st-flash")
    .args(&["write", "tmp/digiblock-hwtest.bin", "0x8000000"])
    .status()
    .await
    .ok()
    .and_then(|res| res.code())*/
}
