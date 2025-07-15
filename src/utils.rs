use crate::protocol;

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

pub fn iterate_position(current_x: f32, current_y: f32, target_x: f32, target_y: f32, speed: f32) -> (f32, f32, f32) {
    let angle = f32::atan2(target_y - current_y, target_x - current_x);
    let next_x = current_x + speed * f32::cos(angle);
    let next_y = current_y + speed * f32::sin(angle);

    return (next_x, next_y, angle);
}

/// Calculate basis functions for NURBS evaluation
fn basis_functions(degree: i64, knot_vector: &[f32], u: f32) -> Vec<f32> {
    let n = knot_vector.len() - degree as usize - 1;
    let mut basis = vec![0.0; n];
    
    // Find the span using binary search for better performance
    let mut span = degree as usize;
    for i in degree as usize..knot_vector.len() - 1 {
        if u >= knot_vector[i] && u < knot_vector[i + 1] {
            span = i;
            break;
        }
    }
    
    // Handle the case where u equals the last knot
    if u >= knot_vector[knot_vector.len() - 1] {
        span = knot_vector.len() - degree as usize - 2;
    }
    
    // Initialize the basis function
    basis[span] = 1.0;
    
    // Compute basis functions using Cox-de Boor recursion
    for k in 1..=degree {
        let mut temp = vec![0.0; n];
        
        for j in span - k as usize..=span {
            if j >= n {
                continue;
            }
            
            let mut saved = 0.0;
            
            // Left basis function
            if basis[j] != 0.0 {
                let denom = knot_vector[j + k as usize] - knot_vector[j];
                if denom != 0.0 {
                    let left = (u - knot_vector[j]) / denom;
                    saved = basis[j] * left;
                }
            }
            
            // Right basis function
            if j < n - 1 && basis[j + 1] != 0.0 {
                let denom = knot_vector[j + k as usize + 1] - knot_vector[j + 1];
                if denom != 0.0 {
                    let right = (knot_vector[j + k as usize + 1] - u) / denom;
                    saved += basis[j + 1] * right;
                }
            }
            
            temp[j] = saved;
        }
        
        // Copy temp back to basis
        for j in 0..n {
            basis[j] = temp[j];
        }
    }
    
    basis
}

/// Evaluate NURBS curve at parameter u and return position and tangent
fn evaluate_nurbs_with_tangent(trajectory: &protocol::vda5050_common::Trajectory, u: f32) -> (f32, f32, f32, f32, f32, bool) {
    let basis = basis_functions(trajectory.degree, &trajectory.knot_vector, u);
    
    let mut x = 0.0;
    let mut y = 0.0;
    let mut theta = 0.0;
    let mut total_weight = 0.0;
    let mut has_explicit_theta = false;
    let mut theta_weight_sum = 0.0;
    
    // Calculate position
    for (i, &weight) in basis.iter().enumerate() {
        if weight > 0.0 {
            let control_point = &trajectory.control_points[i];
            let point_weight = control_point.weight.unwrap_or(1.0);
            let weighted_weight = weight * point_weight;
            
            x += control_point.x * weighted_weight;
            y += control_point.y * weighted_weight;
            
            if let Some(orientation) = control_point.orientation {
                theta += orientation * weighted_weight;
                theta_weight_sum += weighted_weight;
                has_explicit_theta = true;
            }
            
            total_weight += weighted_weight;
        }
    }
    
    if total_weight > 0.0 {
        x /= total_weight;
        y /= total_weight;
        if has_explicit_theta && theta_weight_sum > 0.0 {
            theta /= theta_weight_sum;
        }
    }
    
    // Calculate tangent by evaluating at u + small delta
    let delta = 0.001;
    let u_next = (u + delta).min(1.0);
    let basis_next = basis_functions(trajectory.degree, &trajectory.knot_vector, u_next);
    
    let mut x_next = 0.0;
    let mut y_next = 0.0;
    let mut total_weight_next = 0.0;
    
    for (i, &weight) in basis_next.iter().enumerate() {
        if weight > 0.0 {
            let control_point = &trajectory.control_points[i];
            let point_weight = control_point.weight.unwrap_or(1.0);
            let weighted_weight = weight * point_weight;
            
            x_next += control_point.x * weighted_weight;
            y_next += control_point.y * weighted_weight;
            total_weight_next += weighted_weight;
        }
    }
    
    if total_weight_next > 0.0 {
        x_next /= total_weight_next;
        y_next /= total_weight_next;
    }
    
    // Calculate tangent direction
    let tangent_x = x_next - x;
    let tangent_y = y_next - y;
    let tangent_theta = f32::atan2(tangent_y, tangent_x);
    
    (x, y, theta, tangent_x, tangent_theta, has_explicit_theta)
}

/// Find the closest parameter u on the trajectory to the current position
fn find_closest_parameter(trajectory: &protocol::vda5050_common::Trajectory, current_x: f32, current_y: f32) -> f32 {
    let mut closest_u = 0.0;
    let mut min_distance = f32::INFINITY;
    
    // Sample the curve at regular intervals to find the closest point
    let num_samples = 100;
    for i in 0..=num_samples {
        let u = i as f32 / num_samples as f32;
        let (x, y, _, _, _, _) = evaluate_nurbs_with_tangent(trajectory, u);
        let distance = get_distance(current_x, current_y, x, y);
        
        if distance < min_distance {
            min_distance = distance;
            closest_u = u;
        }
    }
    
    closest_u
}

pub fn iterate_position_with_trajectory(current_x: f32, current_y: f32, target_x: f32, target_y: f32, speed: f32, trajectory: protocol::vda5050_common::Trajectory) -> (f32, f32, f32) {
    // Special case for degree 1 trajectories (straight lines)
    if trajectory.degree == 1 && trajectory.control_points.len() == 2 {
        return iterate_position_with_straight_line(current_x, current_y, target_x, target_y, speed, &trajectory);
    }
    
    // Find the closest point on the trajectory to current position
    let current_u = find_closest_parameter(&trajectory, current_x, current_y);
    
    // Calculate the parameter step based on speed and trajectory length
    // First, estimate the total trajectory length by sampling
    let num_samples = 100; // Increased for better accuracy
    let mut total_length = 0.0;
    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    let mut first_point = true;
    
    for i in 0..=num_samples {
        let u = i as f32 / num_samples as f32;
        let (x, y, _, _, _, _) = evaluate_nurbs_with_tangent(&trajectory, u);
        
        if !first_point {
            total_length += get_distance(prev_x, prev_y, x, y);
        }
        prev_x = x;
        prev_y = y;
        first_point = false;
    }
    
    // Calculate parameter step based on speed and total length
    // This gives us a more accurate step size
    let parameter_step = if total_length > 0.0 {
        let step = speed / total_length;
        // For degree 2 curves, use a more conservative step size
        if trajectory.degree == 2 {
            step.min(0.05) // Smaller step for smoother curves
        } else {
            step.min(0.1)
        }
    } else {
        speed * 0.5 // Fallback if we can't calculate length
    };
    
    // Calculate next parameter value
    let next_u = (current_u + parameter_step).min(1.0);
    
    // Evaluate the trajectory at the next parameter with tangent information
    let (next_x, next_y, explicit_theta, _tangent_x, tangent_theta, has_explicit_theta) = evaluate_nurbs_with_tangent(&trajectory, next_u);
    
    // Determine the final theta
    let final_theta = if has_explicit_theta {
        // Use explicit orientation from control points if available
        explicit_theta
    } else {
        // Use tangential direction of the trajectory
        tangent_theta
    };
    
    // Check if we're close to the target or at the end of trajectory
    let distance_to_target = get_distance(next_x, next_y, target_x, target_y);
    
    // If we're very close to the target, go directly to it
    if distance_to_target <= speed {
        return (target_x, target_y, final_theta);
    }
    
    // If we're at the end of the trajectory but still far from target, move towards target
    if next_u >= 0.99 && distance_to_target > speed {
        let angle = f32::atan2(target_y - next_y, target_x - next_x);
        let final_x = next_x + speed * f32::cos(angle);
        let final_y = next_y + speed * f32::sin(angle);
        return (final_x, final_y, angle);
    }
    
    (next_x, next_y, final_theta)
}

/// Specialized function for straight line trajectories (degree 1 NURBS)
fn iterate_position_with_straight_line(current_x: f32, current_y: f32, target_x: f32, target_y: f32, speed: f32, trajectory: &protocol::vda5050_common::Trajectory) -> (f32, f32, f32) {
    let start_x = trajectory.control_points[0].x;
    let start_y = trajectory.control_points[0].y;
    let end_x = trajectory.control_points[1].x;
    let end_y = trajectory.control_points[1].y;
    
    // Calculate the total length of the line
    let line_length = get_distance(start_x, start_y, end_x, end_y);
    
    // Find the closest point on the line to current position
    let t = if line_length > 0.0 {
        let dot_product = (current_x - start_x) * (end_x - start_x) + (current_y - start_y) * (end_y - start_y);
        (dot_product / (line_length * line_length)).max(0.0).min(1.0)
    } else {
        0.0
    };
    
    // Calculate the current position on the line
    let line_x = start_x + t * (end_x - start_x);
    let line_y = start_y + t * (end_y - start_y);
    
    // Calculate how far to move along the line
    let distance_to_move = speed;
    let new_t = (t + distance_to_move / line_length).min(1.0);
    
    // Calculate new position on the line
    let new_x = start_x + new_t * (end_x - start_x);
    let new_y = start_y + new_t * (end_y - start_y);
    
    // Calculate orientation (tangent to the line)
    let theta = f32::atan2(end_y - start_y, end_x - start_x);
    
    // Check if we're close to the target
    let distance_to_target = get_distance(new_x, new_y, target_x, target_y);
    if distance_to_target <= speed {
        return (target_x, target_y, theta);
    }
    
    (new_x, new_y, theta)
}

pub fn get_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    return ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
}
