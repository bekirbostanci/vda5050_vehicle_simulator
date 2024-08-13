pub fn get_timestamp() -> String {
    //YYYY-MM-DDTHH:mm:ss.ssZ
    let now = chrono::Utc::now();
    return now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
}

pub fn get_topic_type(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((_, file_name)) => file_name,
        _none => path, // if there's no '/' in the path, return the whole path
    }
}

pub fn check_deviation_range(
    node_x: f32,
    node_y: f32,
    vehicle_x: f32,
    vehicle_y: f32,
    deviation_range: f32,
) -> bool {
    let distance = ((node_x - vehicle_x).powi(2) + (node_y - vehicle_y).powi(2)).sqrt();
    return distance <= deviation_range;
}

pub fn iterate_position(current_x: f32, current_y: f32, target_x: f32, target_y: f32, speed: f32) -> (f32, f32, f32) {
    let angle = f32::atan2(target_y - current_y, target_x - current_x);
    let next_x = current_x + speed * f32::cos(angle);
    let next_y = current_y + speed * f32::sin(angle);

    return (next_x, next_y, angle);

}

pub fn get_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    return ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
}