use std::process::{Command, Stdio};

pub fn normalise_url(url: &str) -> Result<String, String> {
    let mut new_url: String = url.trim().to_string();
    if new_url.is_empty() {
        return Err("URL cannot be empty!".to_string());
    }
    if new_url.chars().any(char::is_whitespace) {
        return Err(format!("URL cannot contain whitespace: {}", new_url));
    }
    if !(new_url.contains(".")) {
        return Err(format!("URL is not valid: {}", new_url));
    }
    if !(new_url.starts_with("http://") || new_url.starts_with("https://")) {
        new_url = format!("https://{new_url}");
    }

    Ok(new_url)
}

// there's no outside variables
pub fn sanitise_filename(input: &str) -> String {
    let invalid: [char; 9] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let cleaned: String = input
        .trim()
        .chars()
        .filter(|ch| !invalid.contains(ch) && !ch.is_control())
        .collect();

    let cleaned = cleaned.trim();

    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned.to_string()
    }
}

pub fn check_dependencies() -> Result<(), String> {
    if !command_exists("yt-dlp") {
        return Err("yt-dlp is not installed or not available in PATH.".to_string());
    }
    if !command_exists("ffmpeg") {
        return Err("ffmpeg is not installed or not available in PATH".to_string());
    }
    Ok(())
}

fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn _validate_url(url: &str) -> Result<bool, String> {
    // empty strings
    // check if it starts with http or https
    let url: &str = url.trim();
    if url.is_empty() {
        return Err("Url cannot be empty!".to_string());
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(format!("Url must start with http:// or https://: {}", url));
    }
    Ok(true)
}
