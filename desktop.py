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


def main():
    port = find_available_port(8000)
    server_url = f"http://127.0.0.1:{port}"
    health_url = f"{server_url}/api/health"

    print(f"[ColLink] Starting background API server on port {port}...")

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
        print("[ColLink] Error: Background API server failed to start.")
        server.should_exit = True
        sys.exit(1)

    print(f"[ColLink] API ready at {server_url}. Launching standalone window...")

    browser_exe = get_browser_executable()
    browser_process = None

    if browser_exe:
        # Isolated profile directory to ensure standalone window even if Edge is already open
        profile_dir = os.path.join(ROOT_DIR, "backend", "data", "edge_profile")
        os.makedirs(profile_dir, exist_ok=True)

        cmd = [
            browser_exe,
            f"--app={server_url}",
            f"--user-data-dir={profile_dir}",
            "--window-size=1440,900",
            "--disable-features=TranslateUI",
            "--no-first-run",
            "--no-default-browser-check",
        ]
        browser_process = subprocess.Popen(cmd)
    else:
        # Fallback to default system browser
        webbrowser.open(server_url)

    # Initialize heartbeat state
    APP_STATE["started_at"] = time.time()
    APP_STATE["last_heartbeat"] = time.time()
    APP_STATE["shutdown_requested"] = False

    # Process watchdog loop
    # Exits when:
    # 1. Edge application window process terminates
    # 2. Frontend window emits shutdown beacon (/api/shutdown)
    # 3. Heartbeat lost for > 8 seconds (after initial 15s grace period)
    while True:
        time.sleep(0.5)

        # Condition 1: Browser process exited
        if browser_process is not None and browser_process.poll() is not None:
            print("[ColLink] Window closed by user.")
            break

        # Condition 2: Explicit shutdown request from frontend beforeunload
        if APP_STATE.get("shutdown_requested"):
            print("[ColLink] Shutdown beacon received from frontend.")
            break

        # Condition 3: Heartbeat watchdog
        now = time.time()
        if now - APP_STATE["started_at"] > 15.0:
            if now - APP_STATE["last_heartbeat"] > 8.0:
                print("[ColLink] Heartbeat lost. Window assumed closed.")
                break

    # Clean shutdown
    if browser_process is not None and browser_process.poll() is None:
        try:
            browser_process.terminate()
        except Exception:
            pass

    server.should_exit = True
    print("[ColLink] Exiting cleanly.")
    sys.exit(0)


if __name__ == "__main__":
    main()
