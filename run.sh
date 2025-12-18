#!/bin/bash

# =============================================================================
# ComfyUI Launcher Script
# =============================================================================
# - Stops any previous ComfyUI server instance
# - Closes any existing ComfyUI Brave app window (using unique window class)
# - Starts ComfyUI in its virtual environment
# - Opens a fresh dedicated Brave app window (isolated temporary profile + unique class)
# - Keeps the script alive for logs and clean shutdown (Ctrl+C)
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
COMFYUI_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$COMFYUI_DIR/comfyui.pid"
URL="http://127.0.0.1:8188"
TEMP_PROFILE_DIR="$(mktemp -d /tmp/comfyui-profile.XXXXXX)"  # Unique temp dir each launch
WINDOW_CLASS="ComfyUI-App"                                      # Unique class for reliable identification
SERVER_START_DELAY=8

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------

log() {
    echo "[$(date +'%H:%M:%S')] $*"
}

stop_previous_instance() {
    log "Checking for previous ComfyUI server instance..."

    if [[ -f "$PID_FILE" ]]; then
        local pid
        pid=$(cat "$PID_FILE")

        if kill -0 "$pid" 2>/dev/null; then
            log "Found running server (PID: $pid). Stopping it..."
            kill "$pid"
            sleep 3

            if kill -0 "$pid" 2>/dev/null; then
                log "Process did not stop gracefully. Forcing termination..."
                kill -9 "$pid"
            fi
            log "Previous ComfyUI server stopped."
        else
            log "Stale PID file found (process not running). Cleaning up."
        fi

        rm -f "$PID_FILE"
    else
        log "No previous server instance detected."
    fi
}

close_existing_app_window() {
    if ! command -v wmctrl >/dev/null 2>&1; then
        log "wmctrl not available – skipping window close attempt."
        return
    fi

    log "Checking for existing ComfyUI Brave app window (class: $WINDOW_CLASS)..."

    # Use -x to include window class in output, then match exactly on our custom class
    local windows
    windows=$(wmctrl -l -x | grep -i "$WINDOW_CLASS" | awk '{print $1}')

    if [[ -z "$windows" ]]; then
        log "No existing ComfyUI app window found."
        return
    fi

    log "Found existing ComfyUI window(s). Closing gracefully..."
    for win_id in $windows; do
        # Graceful close (sends close event, like clicking X)
        wmctrl -i -c "$win_id"
    done

    # Allow time for windows to close
    sleep 2
}

activate_venv() {
    local venv_path="$COMFYUI_DIR/python3.12_venv"

    log "Activating virtual environment: $venv_path"
    # shellcheck source=/dev/null
    source "$venv_path/bin/activate" || {
        log "ERROR: Failed to activate virtual environment."
        exit 1
    }
}

open_comfyui_app() {
    log "Launching new dedicated ComfyUI Brave app window..."

    brave-browser \
        --app="$URL" \
        --new-window \
        --user-data-dir="$TEMP_PROFILE_DIR" \
        --class="$WINDOW_CLASS" \
        >/dev/null 2>&1 &

    log "New app window launched (class: $WINDOW_CLASS, profile: $TEMP_PROFILE_DIR)"
}

start_comfyui() {
    log "Starting ComfyUI server..."
    python main.py --enable-manager &

    local pid=$!
    echo "$pid" > "$PID_FILE"
    log "ComfyUI started (PID: $pid)"
}

# -----------------------------------------------------------------------------
# Main Execution
# -----------------------------------------------------------------------------

cd "$COMFYUI_DIR" || { log "ERROR: Cannot access ComfyUI directory: $COMFYUI_DIR"; exit 1; }

stop_previous_instance
close_existing_app_window   # Only closes our isolated ComfyUI app window (safe for other Brave tabs)
activate_venv
start_comfyui

log "Waiting $SERVER_START_DELAY seconds for server to initialize..."
sleep "$SERVER_START_DELAY"

open_comfyui_app

log "ComfyUI is running in a fresh dedicated app window. Press Ctrl+C to stop the server."
wait $!
