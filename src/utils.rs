pub fn get_timestamp() -> String {
    //YYYY-MM-DDTHH:mm:ss.ssZ
    let now = chrono::Utc::now();
    return now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
}