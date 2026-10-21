import os
import sys
import io

# Crucial Windows pythonw.exe fix:
# Under pythonw.exe (GUI mode), sys.stdout and sys.stderr are None.
# Libraries like uvicorn call sys.stdout.isatty() which crashes with:
# AttributeError: 'NoneType' object has no attribute 'isatty' -> ValueError: Unable to configure formatter 'default'
class NullWriter(io.StringIO):
    def write(self, s):
        pass
    def flush(self):
        pass
    def isatty(self):
        return False

if sys.stdout is None:
    sys.stdout = NullWriter()
if sys.stderr is None:
    sys.stderr = NullWriter()

# Force unique AppUserModelID so Windows taskbar loads fresh ColLink icon
try:
    import ctypes
    ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID("ColLink.Studio.Desktop.v2")
except Exception:
    pass

import time
import socket
import logging
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

# File logger with immediate flush
LOG_FILE = os.path.join(ROOT_DIR, "collink.log")

class FlushFileHandler(logging.FileHandler):
    def emit(self, record):
        super().emit(record)
        self.flush()

root_logger = logging.getLogger()
root_logger.setLevel(logging.INFO)
file_handler = FlushFileHandler(LOG_FILE, mode="w", encoding="utf-8")
file_handler.setFormatter(logging.Formatter("%(asctime)s [%(levelname)s] %(message)s"))
root_logger.addHandler(file_handler)
logging.info(f"ColLink launcher started. Python: {sys.executable}, Root: {ROOT_DIR}")


def show_error(message: str, title: str = "ColLink 启动错误"):
    """Show native Windows popup on critical failure."""
    logging.critical(f"FATAL: {message}")
    try:
        import ctypes
        ctypes.windll.user32.MessageBoxW(0, message, title, 0x10)
    except Exception:
        pass


def excepthook(exc_type, exc_value, exc_traceback):
    logging.critical("Uncaught exception:", exc_info=(exc_type, exc_value, exc_traceback))
    show_error(f"启动发生未捕获异常:\n{exc_value}")


sys.excepthook = excepthook

try:
    from app.api.api import APP_STATE
except Exception as e:
    show_error(f"无法导入后端模块 (app.api.api):\n{e}")
    sys.exit(1)


def is_port_bindable(port: int) -> bool:
    """Check if port can be truly bound by a socket."""
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            s.bind(("127.0.0.1", port))
            return True
    except OSError:
        return False


def find_available_port(preferred: int = 8000) -> int:
    """Find a truly bindable port, prioritizing preferred."""
    if is_port_bindable(preferred):
        return preferred
    for p in range(8001, 8099):
        if is_port_bindable(p):
            return p
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def wait_for_server(url: str, timeout: float = 15.0) -> bool:
    """Fast polling until backend server is ready."""
    start = time.time()
    while time.time() - start < timeout:
        try:
            with urllib.request.urlopen(url, timeout=0.5) as resp:
                if resp.status == 200:
                    return True
        except Exception:
            time.sleep(0.1)
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


def apply_window_icon():
    """Find the launched ColLink window and inject WM_SETICON with native .ico."""
    icon_path = os.path.join(ROOT_DIR, "app_icon.ico")
    if not os.path.exists(icon_path):
        return
    try:
        import ctypes
        from ctypes import wintypes
        user32 = ctypes.windll.user32

        WM_SETICON = 0x0080
        ICON_SMALL = 0
        ICON_BIG = 1
        IMAGE_ICON = 1
        LR_LOADFROMFILE = 0x00000010

        hicon_small = user32.LoadImageW(None, icon_path, IMAGE_ICON, 16, 16, LR_LOADFROMFILE)
        hicon_big = user32.LoadImageW(None, icon_path, IMAGE_ICON, 32, 32, LR_LOADFROMFILE)

        if not hicon_small and not hicon_big:
            return

        # Poll for window appearance
        for _ in range(30):
            time.sleep(0.4)
            found_hwnds = []

            def enum_cb(hwnd, extra):
                if user32.IsWindowVisible(hwnd):
                    length = user32.GetWindowTextLengthW(hwnd)
                    if length > 0:
                        buf = ctypes.create_unicode_buffer(length + 1)
                        user32.GetWindowTextW(hwnd, buf, length + 1)
                        if "ColLink" in buf.value:
                            found_hwnds.append(hwnd)
                return True

            WNDENUMPROC = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
            user32.EnumWindows(WNDENUMPROC(enum_cb), 0)

            for hwnd in found_hwnds:
                if hicon_small:
                    user32.SendMessageW(hwnd, WM_SETICON, ICON_SMALL, hicon_small)
                if hicon_big:
                    user32.SendMessageW(hwnd, WM_SETICON, ICON_BIG, hicon_big)

            if found_hwnds:
                logging.info(f"WM_SETICON successfully applied to {len(found_hwnds)} ColLink window(s).")
                break
    except Exception as e:
        logging.warning(f"Could not apply WM_SETICON: {e}")


def launch_browser(url: str) -> None:
    """Launch application in Edge/Chrome standalone app mode, fallback to default browser."""
    browser_exe = get_browser_executable()
    logging.info(f"Detected browser: {browser_exe}")

    if browser_exe:
        # Use ColLink_v2 profile to discard old stale Vite favicons
        profile_dir = os.path.join(os.environ.get("LOCALAPPDATA", ROOT_DIR), "ColLink_v2", "edge_profile")
        os.makedirs(profile_dir, exist_ok=True)

        # Clean stale favicon databases and browser cache
        default_dir = os.path.join(profile_dir, "Default")
        if os.path.exists(default_dir):
            for stale_file in ["Favicons", "Favicons-journal", "Shortcuts", "Shortcuts-journal", "Web Data", "Web Data-journal"]:
                stale_path = os.path.join(default_dir, stale_file)
                if os.path.exists(stale_path):
                    try:
                        os.remove(stale_path)
                    except Exception:
                        pass

        for cache_name in ["Cache", "Code Cache", "GPUCache"]:
            cache_path = os.path.join(profile_dir, "Default", cache_name)
            if os.path.exists(cache_path):
                try:
                    import shutil
                    shutil.rmtree(cache_path, ignore_errors=True)
                except Exception:
                    pass

        cmd = [
            browser_exe,
            f"--user-data-dir={profile_dir}",
            "--no-first-run",
            "--no-default-browser-check",
            "--disable-http-cache",
            f"--app={url}",
            "--window-size=1440,900",
        ]
        logging.info(f"Running browser command: {' '.join(cmd)}")
        try:
            subprocess.Popen(cmd)
            logging.info("Browser subprocess launched successfully.")
            # Trigger native window icon injection in background
            threading.Thread(target=apply_window_icon, daemon=True).start()
            return
        except Exception as e:
            logging.error(f"Failed to launch {browser_exe}: {e}", exc_info=True)

    logging.info("Falling back to default system browser.")
    webbrowser.open(url)


def main():
    try:
        port = find_available_port(8000)
        server_url = f"http://127.0.0.1:{port}"
        health_url = f"{server_url}/api/health"

        logging.info(f"Selected port {port}. Starting Uvicorn...")
        print(f"[ColLink] Starting background API server on port {port}...", flush=True)

        config = uvicorn.Config(
            app="app.main:app",
            host="127.0.0.1",
            port=port,
            log_level="warning",
            log_config=None,
            loop="asyncio",
        )
        server = uvicorn.Server(config)
        server_thread = threading.Thread(target=server.run, daemon=True)
        server_thread.start()

        if not wait_for_server(health_url, timeout=15.0):
            err_msg = f"ColLink 后端服务未能于 15 秒内在端口 {port} 启动就绪。"
            show_error(err_msg)
            server.should_exit = True
            sys.exit(1)

        logging.info(f"API server ready at {server_url}. Launching browser window...")
        print(f"[ColLink] API ready at {server_url}. Opening workspace...", flush=True)

        launch_browser(server_url)

        # Initialize lifecycle state
        start_time = time.time()
        APP_STATE["started_at"] = start_time
        APP_STATE["last_heartbeat"] = start_time
        APP_STATE["has_connected"] = False
        APP_STATE["shutdown_requested_at"] = None

        # Watchdog loop
        while True:
            time.sleep(1.0)
            now = time.time()

            # Check 1: User closed the window (beforeunload beacon received)
            shutdown_req = APP_STATE.get("shutdown_requested_at")
            if shutdown_req is not None:
                if now - shutdown_req >= 5.0:
                    logging.info("Client window closed by user beacon. Exiting...")
                    print("[ColLink] Client window closed. Shutting down server...", flush=True)
                    break

            # Check 2: Active client lost heartbeat for > 20 seconds
            if APP_STATE.get("has_connected"):
                if now - APP_STATE["last_heartbeat"] > 20.0:
                    logging.info("Heartbeat lost for > 20s. Exiting...")
                    print("[ColLink] Heartbeat timeout (>20s). Shutting down server...", flush=True)
                    break
            else:
                # Check 3: Initial connection timeout (90 seconds grace period)
                if now - start_time > 90.0:
                    logging.warning("No client connected within 90s grace period. Exiting...")
                    print("[ColLink] Initial connection timeout. No client connected within 90s.", flush=True)
                    break

        server.should_exit = True
        logging.info("ColLink exited cleanly.")
        sys.exit(0)

    except Exception as e:
        show_error(f"ColLink 运行异常:\n{e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
