import os
from pathlib import Path

# Static Globals
APP_NAME = "zealot"

# Computed Globals
BASE_DIR = Path.home() / f".{APP_NAME}"

# Dynamic Globals
PORT = os.environ.get("PORT", 8080)