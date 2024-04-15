use std::fs;
use std::str;
use std::fs::File;
use std::process::Command;
use std::collections::HashMap;
use std::io::{BufReader, BufRead};
use rand::seq::SliceRandom;

fn load_template(file_name: String) -> Result<Vec<String>, ()> {
    let file_path = format!("template/{}.txt", file_name);
    if let Ok(file) = File::open(&file_path) {
        let reader = BufReader::new(file);
        let mut timestamps = Vec::new();
        for line in reader.lines() {
            if let Ok(line_content) = line {
                if !line_content.is_empty() {
                    timestamps.push(line_content);
                }
            }
        }
        Ok(timestamps)
    } else {
        println!("Failed to open the file.");
        Err(())
    }
}

fn parse_timestamp(timestamp: &str) -> (u32, u32, u32, u32) {
    let parts: Vec<_> = timestamp.split(":").collect();
    let hours = parts[0].parse::<u32>().unwrap_or(0);
    let minutes = parts[1].parse::<u32>().unwrap_or(0);
    let seconds = parts[2].parse::<u32>().unwrap_or(0);
    let milliseconds = parts[3].parse::<u32>().unwrap_or(0);
    (hours, minutes, seconds, milliseconds)
}

fn timestamp_to_seconds(timestamp: &str) -> f64 {
    // Split the timestamp into its components
    let parts: Vec<&str> = timestamp.split(":").collect();

    // Extract hours, minutes, seconds, and milliseconds
    let hours: u32 = parts[0].parse().unwrap_or(0);
    let minutes: u32 = parts[1].parse().unwrap_or(0);
    let seconds: u32 = parts[2].parse().unwrap_or(0);
    let milliseconds: u32 = parts[3].parse().unwrap_or(0);

    // Calculate total seconds
    let total_seconds = hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds as f64 + milliseconds as f64 / 1000.0;
    total_seconds
}

fn extract_seconds(timestamps: Vec<String>) -> Vec<String> {
    // find difference adjacent time from timestamps
    let mut seconds = Vec::new();
    for i in 1..timestamps.len() {
        let (h1, m1, s1, ms1) = parse_timestamp(&timestamps[i - 1]);
        let (h2, m2, s2, ms2) = parse_timestamp(&timestamps[i]);
        
        let total_ms1 = (h1 * 3600 + m1 * 60 + s1) * 1000 + ms1;
        let total_ms2 = (h2 * 3600 + m2 * 60 + s2) * 1000 + ms2;
        
        let diff_ms = total_ms2 - total_ms1;

        let diff_hours = diff_ms / (3600 * 1000);
        let diff_minutes = (diff_ms % (3600 * 1000)) / (60 * 1000);
        let diff_seconds = ((diff_ms % (3600 * 1000)) % (60 * 1000)) / 1000;
        let diff_milliseconds = ((diff_ms % (3600 * 1000)) % (60 * 1000)) % 1000;
        seconds.push(format!("{:02}:{:02}:{:02}.{:03}", diff_hours, diff_minutes, diff_seconds, diff_milliseconds));
    }
    seconds
}

fn list_files_in_directory(folder_path: &str) -> Result<Vec<String>, String> {
    let mut file_paths = Vec::new();

    for entry in fs::read_dir(folder_path).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let file_path = entry.path();

        // Check if the entry is a file (not a directory)
        if file_path.is_file() {
            if let Some(file_name) = file_path.to_str() {
                file_paths.push(file_name.to_string());
            } else {
                return Err("Invalid file path".to_string());
            }
        }
    }
    Ok(file_paths)
}

fn adjust_millisec(seconds: Vec<String>) -> Vec<String> {
    // adjust millisec from capcut to ffmpeg
    let mut new_seconds = Vec::new();
    for second in seconds {
        let parts: Vec<&str> = second.split(":").collect();
        let hours: u32 = parts[0].parse().unwrap_or(0);
        let minutes: u32 = parts[1].parse().unwrap_or(0);
        let seconds: u32 = parts[2].parse().unwrap_or(0);
        let milliseconds = parts[3].parse().unwrap_or(0) as f32;
        let new_ms = (milliseconds/0.03) as u32;
        new_seconds.push(format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, new_ms));
    }
    new_seconds
}

fn random_not_same_last_order(last_val: usize, mut video_part: Vec<usize>) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    video_part.shuffle(&mut rng);
    if last_val != video_part[0] {
        video_part
    } else {
        random_not_same_last_order(last_val, video_part)
    }
}

fn extract_fps(output: &str) -> Option<f32> {
    // Iterate over each line of FFmpeg output
    for line in output.lines() {
        // Check if line contains FPS information
        if let Some(index) = line.find("fps") {
            // Extract FPS value from line
            if let Some(fps_str) = line[..index].split_whitespace().last() {
                // Parse FPS value as f32
                if let Ok(fps) = fps_str.parse::<f32>() {
                    return Some(fps);
                }
            }
        }
    }
    None
}

fn get_video_duration(file_path: &str) -> Result<f32, String> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(file_path)
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if output.status.success() {
        let duration = str::from_utf8(&output.stdout)
            .map_err(|e| format!("Failed to parse output: {}", e))?
            .trim()
            .to_string();
        Ok(duration.parse().unwrap())
    } else {
        Err("ffprobe command failed".to_string())
    }
}

fn seconds_to_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds / 60.0) % 60.0) as u32;
    let seconds_whole = seconds as u32 - minutes * 60;
    let milliseconds = ((seconds - (minutes as f64 *60.0) - (seconds_whole as f64))* 1000.0) as u32;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds_whole, milliseconds)
}

fn add_timestamps(timestamp1: &str, timestamp2: &str, fps: f32) -> String {
    // Function to convert timestamp to total milliseconds
    fn timestamp_to_milliseconds(timestamp: &str) -> u32 {
        let parts: Vec<&str> = timestamp.split(":").collect();
        let hours: u32 = parts[0].parse().unwrap_or(0);
        let minutes: u32 = parts[1].parse().unwrap_or(0);
        let last_parts: Vec<&str> = parts[2].split(".").collect();
        let seconds: u32 = last_parts[0].parse().unwrap_or(0);
        let milliseconds: u32 = last_parts[1].parse().unwrap_or(0);
        hours * 3600_000 + minutes * 60_000 + seconds * 1000 + milliseconds
    }

    // Function to convert total milliseconds to timestamp string
    fn milliseconds_to_timestamp(milliseconds: u32, fps: f32) -> String {
        let hours = milliseconds / 3600_000;
        let minutes = (milliseconds % 3600_000) / 60_000;
        let mut seconds = (milliseconds % 60_000) / 1000;
        let mut milliseconds = milliseconds % 1000;
        if (fps * 100.0) % 100.0 != 0.0 {
            let s_ms = (seconds*1000) + milliseconds -50;
            seconds = (s_ms/1000) as u32;
            milliseconds = (s_ms%1000) as u32;
        }
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
    }
    
    // Convert timestamps to total milliseconds, add them together, and then convert back to timestamp
    let total_milliseconds =
        timestamp_to_milliseconds(timestamp1) + timestamp_to_milliseconds(timestamp2);

    milliseconds_to_timestamp(total_milliseconds, fps)
}

// ffmpeg -i a.mp4 -ss 00:01:02.500 -t 00:01:03.250 -c copy x2.mp4
// start, duration
fn main() {
    let music_theme = String::from("2phut_hon");
    let seconds = load_template(music_theme.clone()).unwrap();
    let seconds: Vec<String> = adjust_millisec(seconds);
    // let seconds = extract_seconds(timestamps);
    let n_section = seconds.len();
    let videos_path = list_files_in_directory("videos").unwrap();
    // fps: affect of ffmpeg cut seconds: if not round (29.97) -> -= 50ms
    let mut fps_arr = Vec::new();
    for video_path in &videos_path {
        let output = Command::new("ffmpeg")
            .args(&["-i", video_path])
            .output().unwrap();
        let out_str = String::from_utf8_lossy(&output.stderr);
        let fps = extract_fps(&out_str).unwrap();
        fps_arr.push(fps);
    }
    let n_videos = videos_path.len();
    println!("\nLoaded Template: {}", music_theme);

    let video_part: Vec<usize> = (0..n_videos).collect();
    let mut part_orders: Vec<usize> = vec![];
    part_orders.extend(video_part.iter()); // initail values
    // push random orders
    while part_orders.len() < n_section {
        let last_idx = part_orders.len()-1;
        let last_val = part_orders[last_idx];
        let video_part = random_not_same_last_order(last_val, video_part.clone());
        part_orders.extend(video_part.iter());
    }
    // remove over elements
    part_orders = part_orders[0..n_section].to_owned();
    // elem: count
    let mut n_parts: HashMap<usize, usize> = HashMap::new();
    for n in &part_orders {
        let count = n_parts.entry(*n).or_insert(0);
        *count += 1;
    }

    // find right section for cut each video
    let mut details: HashMap<usize, Vec<f64>> = HashMap::new();
    for idx in 0..n_videos {
        details.insert(idx, vec![]);
    }
    let reduction = 0.4; // 20% cut edge 
    for (idx, video_path) in videos_path.iter().enumerate() {
        let duration = get_video_duration(&video_path).unwrap();
        let iterval = (duration*(1.0-reduction)/n_parts[&idx] as f32) as u32 as f32;
        // ffmpeg bug no video if start before 10s
        let mut start_val = (duration*(reduction/2.0)) as u32 as f32;
        for _ in 0..n_parts[&idx] {
            details.get_mut(&idx).unwrap()
            .push(start_val as f64);
            start_val += iterval;
        }
    }

    // split video from part_orders
    let mut usage: HashMap<usize, usize> = HashMap::new();
    for idx in 0..n_videos {
        usage.insert(idx, 0);
    }
    // args 
    println!("Spliting Videos");
    fs::create_dir_all("temp_videos").unwrap();
    for (section, order) in part_orders.iter().enumerate() {
        let video_path = &videos_path[*order];
        let use_idx = usage.entry(*order).or_insert(0);
        // create command 
        let start_second = details.get(order).unwrap()[*use_idx];
        let ss = seconds_to_time(start_second);
        let t = &seconds[section];
        let to = add_timestamps(&ss, t, fps_arr[*order]);
        let dest_path = format!("temp_videos/{}.mp4", section+1);
        let args = [
            "-i",
            video_path,
            "-ss",
            &ss,
            "-to",
            &to,
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-strict",
            "experimental",
            "-async",
            "1",
            &dest_path
        ];

        let res = Command::new("ffmpeg")
            .args(args)
            .output()
            .expect("Failed to execute command");

        // prettier print
        let max_length = videos_path.iter().map(|s| s.len()).max().unwrap_or(0) + 1;
        if res.status.success() {
            println!(" {:<02}/{} : {:<width$} {} -> {} | {} ",section+1, n_section, video_path, ss, to, t, width = max_length);
        } else {
            // Print error message if command failed
            println!("Error: {}", String::from_utf8_lossy(&res.stderr));
        }
        *use_idx += 1; // update idx
    }
}
