use std::fs;
use std::process::Command;
use std::fs::File;
use std::io::Write;
// use std::io;
use std::thread;
use std::time::Duration;

// sort file name ascending order
fn extract_number(file_name: &str) -> u32 {
    file_name
        .chars()
        .filter(|c| c.is_digit(10))
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

pub fn combined_videos(folder_name: &str) -> Result<(), std::io::Error>{
    let dir_path: Result<Vec<_>, _> = fs::read_dir(format!("./{}", folder_name))?.collect();
    let file_names: Vec<String> = dir_path?
                                .iter()
                                .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
                                .collect();
    // file list
    let mut file_names: Vec<&str> = file_names.iter().map(|name| name.as_str()).collect();
    file_names.sort_by(|a, b| {
        let num_a = extract_number(a);
        let num_b = extract_number(b);
        num_a.cmp(&num_b)
    });

    //write .txt for path list
    let mut file = File::create(format!("{}.txt", folder_name))?;
    for file_name in file_names {
        let filepath = format!("{}/{}", folder_name, file_name);
        // check is actual video file
        if let Ok(metadata) = fs::metadata(filepath.as_str()) {
            let file_size = metadata.len();
            if file_size > 10000 {
                writeln!(file, "file '{}'", filepath)?;
            }
        }
    }

    // combined files
    let output = Command::new("ffmpeg")
                                .args(["-f", "concat", "-safe", "0", "-i", &format!("{}.txt", folder_name), "-c", "copy", &format!("{} - concated.mp4", folder_name)])
                                .output()
                                .expect("failed to execute process");

    let _ = output.stdout;
    Ok(())
}

fn concat_and_remove(user_id: &str) {
        let res = combined_videos(&user_id);
        _ = fs::remove_file(&format!("{}.txt", user_id));
        match res {
            Ok(()) => {
                println!("\ncomplete: {} - concated", user_id);
                thread::sleep(Duration::from_secs(5));
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

fn main() {
    // let mut user_id_input = String::new();
    // println!("Enter folder_name for concat :");
    // io::stdin().read_line(&mut user_id_input).expect("Failed to read");
    let user_id = "temp_videos";

    if user_id == "./" {
        let dir_path: Result<Vec<_>, _> = fs::read_dir(".").unwrap().collect();
        let file_names: Vec<String> = dir_path.unwrap()
                                .iter()
                                .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
                                .collect();
        
        for path in &file_names {
            let path = path.as_str();
            if !(path.contains(".exe") | path.contains(".toml") | path.contains(".lock") | path.contains(".mp4")  | (path == "src") | (path == "target")) {
                concat_and_remove(&path);
            }
        }
    } else {
        concat_and_remove(&user_id);
    }
}