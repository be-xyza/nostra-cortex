#!/usr/bin/env python3
"""Shared WorkRouter filesystem path helpers."""

from __future__ import annotations

import os
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def work_router_log_root() -> Path:
    configured = os.environ.get("WORK_ROUTER_LOG_ROOT", "").strip()
    if configured:
        return Path(configured)
    return ROOT / "logs" / "work_router"

