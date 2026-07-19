#!/usr/bin/env python3
import json
import os
import subprocess
import sys

data = json.load(sys.stdin)
file_path = data.get("tool_input", {}).get("file_path", "")
if file_path.endswith(".rs"):
    repo_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    subprocess.run(["cargo", "fmt", "--", file_path], cwd=repo_root)
