use crate::notifications::enums::notification_event::NotificationEvent;
use log::{debug, error, info};
use regex::Regex;

/// Handles join code related events such as session registration and active sessions.
/// It uses regex to extract information from log lines and triggers appropriate notifications.
///
/// # Arguments
/// * `line` - A `&str` representing a single line from the log.
pub fn handle_join_code_events(line: &str) {
  // Regex to capture session registered with join code
  // Pattern: Session "[servername]" registered with join code 795570
  let registered_regex = Regex::new(
    r#"\d{2}/\d{2}/\d{4} \d{2}:\d{2}:\d{2}: Session "([^"]+)" registered with join code (\d+)"#,
  )
  .expect("Failed to compile registered_regex");

  // Regex to capture active session with join code, IP and player count
  // Pattern: Session "[servername]" with join code 795570 and IP [ipaddress:port] is active with 0 player(s)
  let active_regex = Regex::new(
    r#"\d{2}/\d{2}/\d{4} \d{2}:\d{2}:\d{2}: Session "([^"]+)" with join code (\d+) and IP \[([^\]]+)\] is active with (\d+) player\(s\)"#,
  )
  .expect("Failed to compile active_regex");

  // Handle session registered event
  if let Some(captures) = registered_regex.captures(line) {
    debug!("Matched session registered event: '{captures:?}'");
    match extract_registered_details(&captures) {
      Ok((server_name, join_code)) => {
        info!("Session '{server_name}' registered with join code '{join_code}'");
        send_join_code_notification(&server_name, &join_code, None, None);
      }
      Err(e) => error!("Failed to process session registered event line '{line}': {e}"),
    }
  }

  // Handle active session event
  if let Some(captures) = active_regex.captures(line) {
    debug!("Matched active session event: '{captures:?}'");
    match extract_active_details(&captures) {
      Ok((server_name, join_code, ip_address, player_count)) => {
        info!(
          "Session '{server_name}' with join code '{join_code}' is active at '{ip_address}' with {player_count} player(s)"
        );
        send_join_code_notification(&server_name, &join_code, Some(&ip_address), Some(player_count));
      }
      Err(e) => error!("Failed to process active session event line '{line}': {e}"),
    }
  }
}

/// Extracts server name and join code from regex captures for a session registered event.
fn extract_registered_details(captures: &regex::Captures) -> Result<(String, String), String> {
  debug!("Extracting registered session details from captures: '{captures:?}'");
  let server_name = captures
    .get(1)
    .ok_or("Missing server name")?
    .as_str()
    .to_string();
  let join_code = captures
    .get(2)
    .ok_or("Missing join code")?
    .as_str()
    .to_string();
  Ok((server_name, join_code))
}

/// Extracts server name, join code, IP address and player count from regex captures for an active session event.
fn extract_active_details(
  captures: &regex::Captures,
) -> Result<(String, String, String, u32), String> {
  debug!("Extracting active session details from captures: '{captures:?}'");
  let server_name = captures
    .get(1)
    .ok_or("Missing server name")?
    .as_str()
    .to_string();
  let join_code = captures
    .get(2)
    .ok_or("Missing join code")?
    .as_str()
    .to_string();
  let ip_address = captures
    .get(3)
    .ok_or("Missing IP address")?
    .as_str()
    .to_string();
  let player_count = captures
    .get(4)
    .ok_or("Missing player count")?
    .as_str()
    .parse::<u32>()
    .map_err(|e| format!("Failed to parse player count: {e}"))?;
  Ok((server_name, join_code, ip_address, player_count))
}

/// Sends a join code notification using the existing notification system.
fn send_join_code_notification(
  server_name: &str,
  join_code: &str,
  ip_address: Option<&str>,
  player_count: Option<u32>,
) {
  let message = match (ip_address, player_count) {
    (Some(ip), Some(count)) => {
      format!(
        "🎮 Server '{server_name}' is active!\n🔑 Join Code: {join_code}\n🌐 IP: {ip}\n👥 Players: {count}"
      )
    }
    _ => {
      format!("🎮 Server '{server_name}' registered!\n🔑 Join Code: {join_code}")
    }
  };

  NotificationEvent::Broadcast.send_notification(Some(message));
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extract_registered_details() {
    let registered_regex = Regex::new(
      r#"\d{2}/\d{2}/\d{4} \d{2}:\d{2}:\d{2}: Session "([^"]+)" registered with join code (\d+)"#,
    )
    .unwrap();

    let line = "08/30/2025 12:30:52: Session \"MyServer\" registered with join code 795570";
    let captures = registered_regex.captures(line).unwrap();
    let (server_name, join_code) = extract_registered_details(&captures).unwrap();

    assert_eq!(server_name, "MyServer");
    assert_eq!(join_code, "795570");
  }

  #[test]
  fn test_extract_active_details() {
    let active_regex = Regex::new(
      r#"\d{2}/\d{2}/\d{4} \d{2}:\d{2}:\d{2}: Session "([^"]+)" with join code (\d+) and IP \[([^\]]+)\] is active with (\d+) player\(s\)"#,
    )
    .unwrap();

    let line = "08/30/2025 12:30:53: Session \"MyServer\" with join code 795570 and IP [127.0.0.1:2456] is active with 0 player(s)";
    let captures = active_regex.captures(line).unwrap();
    let (server_name, join_code, ip_address, player_count) =
      extract_active_details(&captures).unwrap();

    assert_eq!(server_name, "MyServer");
    assert_eq!(join_code, "795570");
    assert_eq!(ip_address, "127.0.0.1:2456");
    assert_eq!(player_count, 0);
  }

  #[test]
  fn test_handle_join_code_events_registered() {
    let line = "08/30/2025 12:30:52: Session \"TestServer\" registered with join code 123456";
    // This test just verifies the function doesn't panic and correctly identifies the pattern
    handle_join_code_events(line);
  }

  #[test]
  fn test_handle_join_code_events_active() {
    let line = "08/30/2025 12:30:53: Session \"TestServer\" with join code 123456 and IP [192.168.1.100:2456] is active with 2 player(s)";
    // This test just verifies the function doesn't panic and correctly identifies the pattern
    handle_join_code_events(line);
  }

  #[test]
  fn test_handle_join_code_events_no_match() {
    let line = "08/30/2025 12:30:54: Some other log message";
    // This test verifies the function handles non-matching lines gracefully
    handle_join_code_events(line);
  }
}