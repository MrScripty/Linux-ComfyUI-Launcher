# Troubleshooting Guide

## Install Dialog Shows "No Versions Available"

### Symptom
When you click the download button to open the Install Dialog, it shows "No versions available" even though versions should be available from GitHub.

### Root Cause
This issue occurs when the Python version manager fails to initialize due to incorrect module import paths. The terminal will show:
```
Warning: Version management initialization failed: No module named 'backend'
```

### Solution

**DO NOT RUN THE APP LIKE THIS:**
```bash
# ❌ WRONG - This breaks Python imports
python3 backend/main.py
```

**USE ONE OF THESE METHODS INSTEAD:**

#### Method 1: Use the run_app.py wrapper (RECOMMENDED)
```bash
cd "/media/jeremy/OrangeCream/Linux Software/ComfyUI-master/Linux-ComfyUI-Launcher"
python3 run_app.py
```

#### Method 2: Run as a Python module
```bash
cd "/media/jeremy/OrangeCream/Linux Software/ComfyUI-master/Linux-ComfyUI-Launcher"
python3 -m backend.main
```

### Why This Happens

When you run `python3 backend/main.py`, Python:
1. Sets the working directory to the project root
2. But adds `backend/` to the module search path
3. When `main.py` tries to import `from backend.api import ...`, Python can't find the `backend` package

When you run `python3 run_app.py` or `python3 -m backend.main`, Python:
1. Sets the working directory correctly
2. Adds the project root to the module search path
3. All `from backend.X import ...` statements work correctly

### Verifying It Works

After launching with the correct method, you should see:

**In the terminal:**
```
[DEBUG] get_available_versions called (force_refresh=False)
[DEBUG] Retrieved 30 versions from backend
```

**In the developer console (F12):**
```
Total available: 30 Filtered: 30 showPreReleases: true showInstalled: true
```

**In the UI:**
- The Install Dialog should show a list of ComfyUI versions
- You should see version numbers like "v0.5.1", "v0.4.0", etc.
- The dialog should show "30 versions" (or similar) at the top

### Quick Diagnostic

Run this command to verify your backend is working:
```bash
cd "/media/jeremy/OrangeCream/Linux Software/ComfyUI-master/Linux-ComfyUI-Launcher"
python3 diagnose_imports.py
```

If you see "✓ SUCCESS" messages and "Retrieved 30 versions", your backend is working correctly. The issue is just how you're launching the app.

### Still Having Issues?

If you're using the correct launch method and still seeing problems:

1. Check that you're in the correct directory:
   ```bash
   pwd
   # Should output: /media/jeremy/OrangeCream/Linux Software/ComfyUI-master/Linux-ComfyUI-Launcher
   ```

2. Verify the frontend is built:
   ```bash
   ls -la frontend/dist/index.html
   # Should show the file exists
   ```

3. Check for any error messages in:
   - The terminal where you launched the app
   - The browser developer console (F12)

4. Try force-refreshing versions from GitHub by clicking the refresh icon in the VersionSelector
