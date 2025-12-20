#!/usr/bin/env python3
"""
Launcher wrapper that ensures proper Python path setup
"""
import sys
from pathlib import Path

# Add the project root to Python path
project_root = Path(__file__).parent
sys.path.insert(0, str(project_root))

# Now import and run the main application
from backend.main import main

if __name__ == "__main__":
    main()
