import os
import sys
import time
import socket
import threading
import urllib.request
import uvicorn
import webview

# Ensure root directory and backend are in sys.path
ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
BACKEND_DIR = os.path.join(ROOT_DIR, "backend")

if BACKEND_DIR not in sys.path:
    sys.path.insert(0, BACKEND_DIR)

os.chdir(ROOT_DIR)


def find_available_port(preferred: int = 8000) -> int:
    """Find an available port, trying the preferred one first."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        # If connect_ex returns non-zero, port is not currently bound
        if s.connect_ex(("127.0.0.1", preferred)) != 0:
            return preferred

    # Port is occupied, request an ephemeral available port
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def wait_for_server(url: str, timeout: float = 15.0) -> bool:
    """Poll health endpoint until backend server is ready."""
    start = time.time()
    while time.time() - start < timeout:
        try:
            with urllib.request.urlopen(url, timeout=1.0) as resp:
                if resp.status == 200:
                    return True
        except Exception:
            time.sleep(0.1)
    return False


def main():
    port = find_available_port(8000)
    server_url = f"http://127.0.0.1:{port}"
    health_url = f"{server_url}/api/health"

    print(f"[ColLink Desktop] Starting background API server on port {port}...")

    # Configure Uvicorn server
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

    # Wait for server to initialize
    if not wait_for_server(health_url, timeout=15.0):
        print("[ColLink Desktop] Error: Background API server failed to start within timeout.")
        server.should_exit = True
        sys.exit(1)

    print(f"[ColLink Desktop] API server is ready at {server_url}. Launching desktop window...")

    # Create PyWebView desktop window
    window = webview.create_window(
        title="ColLink - 数据字段血缘与可视化映射平台",
        url=server_url,
        width=1440,
        height=900,
        min_size=(1024, 680),
        resizable=True,
        text_select=True,
    )

    # Start PyWebView GUI (blocks until user closes the window)
    # private_mode=False preserves localStorage across application runs
    webview.start(private_mode=False)

    print("[ColLink Desktop] Window closed by user. Exiting application...")
    server.should_exit = True
    sys.exit(0)


if __name__ == "__main__":
    main()
