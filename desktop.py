import os
import sys
import time
import socket
import threading
import subprocess
import webbrowser
import urllib.request
import uvicorn

# Root directory setup
ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
BACKEND_DIR = os.path.join(ROOT_DIR, "backend")

if BACKEND_DIR not in sys.path:
    sys.path.insert(0, BACKEND_DIR)

os.chdir(ROOT_DIR)

# Import backend app state for lifecycle management
from app.api.api import APP_STATE


def find_available_port(preferred: int = 8000) -> int:
    """Find an available port, preferring the default one."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        if s.connect_ex(("127.0.0.1", preferred)) != 0:
            return preferred

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def wait_for_server(url: str, timeout: float = 10.0) -> bool:
    """Fast polling until backend server is ready."""
    start = time.time()
    while time.time() - start < timeout:
        try:
            with urllib.request.urlopen(url, timeout=0.5) as resp:
                if resp.status == 200:
                    return True
        except Exception:
            time.sleep(0.05)
    return False


def get_browser_executable() -> str | None:
    """Find installed Microsoft Edge or Google Chrome executable."""
    candidates = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        os.path.expandvars(r"%LOCALAPPDATA%\Microsoft\Edge\Application\msedge.exe"),
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        os.path.expandvars(r"%LOCALAPPDATA%\Google\Chrome\Application\chrome.exe"),
    ]
    for path in candidates:
        if os.path.exists(path):
            return path
    return None


def launch_browser(url: str) -> None:
    """Launch application in Edge/Chrome standalone app mode, fallback to default browser."""
    browser_exe = get_browser_executable()
    if browser_exe:
        # Dedicated user-data-dir in AppData prevents ProcessSingleton lock collision (Error code: 32)
        # when Edge is already running on the machine, while --no-first-run prevents setup wizards.
        profile_dir = os.path.join(os.environ.get("LOCALAPPDATA", ROOT_DIR), "ColLink", "edge_profile")
        os.makedirs(profile_dir, exist_ok=True)

        cmd = [
            browser_exe,
            f"--user-data-dir={profile_dir}",
            "--no-first-run",
            "--no-default-browser-check",
            f"--app={url}",
            "--window-size=1440,900",
        ]
        try:
            subprocess.Popen(cmd)
            print(f"[ColLink] Launched app window via {os.path.basename(browser_exe)}.", flush=True)
            return
        except Exception as e:
            print(f"[ColLink] Warning: Failed to launch via {browser_exe}: {e}", flush=True)

    print("[ColLink] Launching in default system browser...", flush=True)
    webbrowser.open(url)


def main():
    port = find_available_port(8000)
    server_url = f"http://127.0.0.1:{port}"
    health_url = f"{server_url}/api/health"

    print(f"[ColLink] Starting background API server on port {port}...", flush=True)

    # Configure and launch lightweight Uvicorn server in a daemon thread
    config = uvicorn.Config(
        app="app.main:app",
        host="127.0.0.1",
        port=port,
        log_level="warning",
        loop="asyncio",
    )
    server = uvicorn.Server(config)
    server_thread = threading.Thread(target=server.run, daemon=True)
    server_thread.start()

    # Fast wait for server
    if not wait_for_server(health_url, timeout=10.0):
        print("[ColLink] Error: Background API server failed to start within timeout.", flush=True)
        server.should_exit = True
        sys.exit(1)

    print(f"[ColLink] API ready at {server_url}. Opening workspace...", flush=True)

    # Launch browser window
    launch_browser(server_url)

    # Initialize lifecycle state
    start_time = time.time()
    APP_STATE["started_at"] = start_time
    APP_STATE["last_heartbeat"] = start_time
    APP_STATE["has_connected"] = False
    APP_STATE["shutdown_requested_at"] = None

    # Process watchdog loop
    # NOTE: Chromium launches delegate to existing master processes and their helper
    # launchers exit immediately. We DO NOT monitor the subprocess exit code.
    # Instead, we rely strictly on frontend heartbeats & the beforeunload shutdown beacon.
    while True:
        time.sleep(1.0)
        now = time.time()

        # Check 1: User closed the window (beforeunload beacon received)
        # Give a 5-second grace window to allow for potential page reloads (F5)
        shutdown_req = APP_STATE.get("shutdown_requested_at")
        if shutdown_req is not None:
            if now - shutdown_req >= 5.0:
                print("[ColLink] Client window closed. Shutting down server...", flush=True)
                break

        # Check 2: Active client lost heartbeat for > 15 seconds
        if APP_STATE.get("has_connected"):
            if now - APP_STATE["last_heartbeat"] > 15.0:
                print("[ColLink] Heartbeat timeout (>15s). Shutting down server...", flush=True)
                break
        else:
            # Check 3: Initial connection timeout (60 seconds)
            if now - start_time > 60.0:
                print("[ColLink] Initial connection timeout. No client connected within 60s.", flush=True)
                break

    # Clean shutdown
    server.should_exit = True
    print("[ColLink] Exiting cleanly.", flush=True)
    sys.exit(0)


if __name__ == "__main__":
    main()
