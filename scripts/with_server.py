#!/usr/bin/env python3
from __future__ import annotations

import argparse
import atexit
import shlex
import socket
import subprocess
import sys
import time
from pathlib import Path


def wait_for_port(port: int, timeout_secs: float) -> None:
    deadline = time.time() + timeout_secs
    while time.time() < deadline:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(1.0)
            if sock.connect_ex(("127.0.0.1", port)) == 0:
                return
        time.sleep(0.5)
    raise TimeoutError(f"timed out waiting for port {port}")


def terminate_processes(processes: list[subprocess.Popen[bytes]]) -> None:
    for proc in reversed(processes):
        if proc.poll() is not None:
            continue
        proc.terminate()
    deadline = time.time() + 10
    for proc in reversed(processes):
        if proc.poll() is not None:
            continue
        remaining = max(0.0, deadline - time.time())
        try:
            proc.wait(timeout=remaining)
        except subprocess.TimeoutExpired:
            proc.kill()


def main() -> int:
    parser = argparse.ArgumentParser(description="Run a command with temporary background servers.")
    parser.add_argument("--timeout", type=int, default=180, help="seconds to wait for each port")
    parser.add_argument(
        "--server",
        action="append",
        default=[],
        help="shell command to launch a background server; repeat for each server",
    )
    parser.add_argument(
        "--port",
        action="append",
        type=int,
        default=[],
        help="port that should become reachable for the matching server; repeat in order",
    )
    parser.add_argument("command", nargs=argparse.REMAINDER, help="command to run after --")
    args = parser.parse_args()

    command = list(args.command)
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        parser.error("missing command after --")
    if len(args.server) != len(args.port):
        parser.error("--server and --port must be provided the same number of times")

    processes: list[subprocess.Popen[bytes]] = []
    atexit.register(terminate_processes, processes)

    try:
        for server_cmd, port in zip(args.server, args.port):
            proc = subprocess.Popen(server_cmd, shell=True)
            processes.append(proc)
            wait_for_port(port, float(args.timeout))

        completed = subprocess.run(command, check=False)
        return completed.returncode
    finally:
        terminate_processes(processes)


if __name__ == "__main__":
    raise SystemExit(main())
