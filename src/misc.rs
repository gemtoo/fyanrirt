pub fn die(reason: impl Into<String>) -> ! {
    error!("{}", reason.into());
    std::process::exit(1);
}
pub async fn sleep(millis: u64) {
    let time = std::time::Duration::from_millis(millis);
    tokio::time::sleep(time).await;
}

pub async fn ratelimit(rate_per_second: u64) {
    let time = std::time::Duration::from_millis(1000 / rate_per_second);
    tokio::time::sleep(time).await;
}
