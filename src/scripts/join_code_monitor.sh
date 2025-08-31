#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

# Source utility functions if available
if [ -f "/home/steam/scripts/utils.sh" ]; then
  source "/home/steam/scripts/utils.sh"
fi

# Logging via odin
log() { odin log --message "$*"; }
log_debug() { odin log --level debug --message "$*"; }

# Function to send webhook notification
send_webhook() {
  local message="$1"
  
  if [ -z "$WEBHOOK_URL" ]; then
    log_debug "No webhook URL configured, skipping notification"
    return
  fi

  # Create JSON payload for Discord webhook
  local json_payload
  json_payload=$(cat <<EOF
{
  "content": "$message"
}
EOF
)

  # Send webhook notification
  if curl -s -H "Content-Type: application/json" -d "$json_payload" "$WEBHOOK_URL" >/dev/null 2>&1; then
    log_debug "Join code notification sent successfully"
  else
    log "Failed to send join code notification"
  fi
}

# Function to parse session registered log line
parse_session_registered() {
  local line="$1"
  
  # Extract server name and join code using sed
  # Pattern: Session "[servername]" registered with join code XXXXXX
  if echo "$line" | grep -q 'Session ".*" registered with join code [0-9]\{6\}'; then
    local server_name
    local join_code
    
    server_name=$(echo "$line" | sed -n 's/.*Session "\([^"]*\)" registered with join code [0-9]\{6\}.*/\1/p')
    join_code=$(echo "$line" | sed -n 's/.*registered with join code \([0-9]\{6\}\).*/\1/p')
    
    if [ -n "$server_name" ] && [ -n "$join_code" ]; then
      local message="🎮 Server '$server_name' registered! 🔑 Join Code: $join_code"
      log "Session '$server_name' registered with join code '$join_code'"
      send_webhook "$message"
    fi
  fi
}

# Function to parse session active log line  
parse_session_active() {
  local line="$1"
  
  # Extract server name, join code, IP and player count using sed
  # Pattern: Session "[servername]" with join code XXXXXX and IP [ipaddress:port] is active with X player(s)
  if echo "$line" | grep -q 'Session ".*" with join code [0-9]\{6\} and IP \[.*\] is active with [0-9]* player(s)'; then
    local server_name
    local join_code
    local ip_address
    local player_count
    
    server_name=$(echo "$line" | sed -n 's/.*Session "\([^"]*\)" with join code [0-9]\{6\} and IP \[.*\] is active with [0-9]* player(s).*/\1/p')
    join_code=$(echo "$line" | sed -n 's/.*with join code \([0-9]\{6\}\) and IP \[.*\] is active with [0-9]* player(s).*/\1/p')
    ip_address=$(echo "$line" | sed -n 's/.*and IP \[\([^]]*\)\] is active with [0-9]* player(s).*/\1/p')
    player_count=$(echo "$line" | sed -n 's/.*is active with \([0-9]*\) player(s).*/\1/p')
    
    if [ -n "$server_name" ] && [ -n "$join_code" ] && [ -n "$ip_address" ] && [ -n "$player_count" ]; then
      local message="🎮 Server '$server_name' is active! 🔑 Join Code: $join_code 🌐 IP: $ip_address 👥 Players: $player_count"
      log "Session '$server_name' with join code '$join_code' is active at '$ip_address' with $player_count player(s)"
      send_webhook "$message"
    fi
  fi
}

# Function to monitor log files for join code events
monitor_logs() {
  local log_location="${LOG_LOCATION:-/home/steam/valheim/logs}"
  
  log "Starting join code monitoring on log directory: $log_location"
  
  # Create log directory if it doesn't exist
  mkdir -p "$log_location"
  
  # Monitor all .log files in the log directory
  if ! command -v inotifywait >/dev/null 2>&1; then
    log "inotifywait not available, using tail fallback method"
    
    # Fallback: Use tail to follow all .log files
    # This will follow existing files and periodically check for new ones
    while true; do
      for log_file in "$log_location"/*.log; do
        if [ -f "$log_file" ]; then
          tail -F -n 0 "$log_file" 2>/dev/null | while IFS= read -r line; do
            parse_session_registered "$line"
            parse_session_active "$line"
          done &
        fi
      done
      sleep 5
    done
  else
    # Use inotifywait to monitor for file changes
    inotifywait -m -r -e modify,create "$log_location" --format '%w%f' 2>/dev/null | while read -r file; do
      if [[ "$file" == *.log ]]; then
        # Read new lines from the file
        tail -F -n 0 "$file" 2>/dev/null | while IFS= read -r line; do
          parse_session_registered "$line"
          parse_session_active "$line"  
        done &
      fi
    done
  fi
}

# Main execution
main() {
  # Check if join code notifications are enabled
  if [ "${JOIN_CODE_NOTIFICATIONS:-0}" != "1" ]; then
    log_debug "Join code notifications disabled, exiting"
    exit 0
  fi
  
  # Check if webhook URL is configured
  if [ -z "$WEBHOOK_URL" ]; then
    log "Join code notifications enabled but no webhook URL configured"
    exit 0
  fi
  
  log "Join code notifications enabled, starting monitor"
  monitor_logs
}

# Run main function
main "$@"