#!/usr/bin/env python3
"""
File bundler that watches an input file and clipboard for triggers,
extracts file paths, concatenates their contents, and copies to clipboard.

run this from the root dir re: relative file paths

eg

python scripts/file_bundler/file_bundler.py 

"""

import os
import re
import time
import pyperclip
from pathlib import Path
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

# ============================================================================
# CONFIGURATION
# ============================================================================

INPUT_FILE = "scripts/file_bundler/input.txt"
OUTPUT_FILE = "scripts/file_bundler/output.txt"
CLIPBOARD_ACTIVATION_SIGNAL = "b:go"
CLIPBOARD_POLL_INTERVAL = 0.5  # seconds
FILE_CHANGE_DEBOUNCE = 0.1  # seconds
EXECUTION_LOG_MARKER = "=== BUNDLER RESULTS ==="

# ============================================================================
# PATH EXTRACTION
# ============================================================================

def extract_file_paths(content):
    """
    Extract file paths from content.
    A path is: no spaces, contains '/', ends with an extension (e.g., .py, .rs, .txt)
    Excludes markdown formatting characters like backticks.
    """
    # Pattern: file path characters (not whitespace or backticks) containing '/' and ending with .extension
    pattern = r'[^\s`]+\/[^\s`]+\.\w+'
    matches = re.findall(pattern, content)
    return matches


# ============================================================================
# FILE PROCESSING
# ============================================================================

def resolve_path(path_str, base_dir):
    """
    Resolve a path string to an absolute Path object.
    Relative paths are resolved from base_dir (script execution location).
    """
    path = Path(path_str)
    if path.is_absolute():
        return path
    else:
        return (base_dir / path).resolve()


def read_file_safe(path):
    """
    Attempt to read a file. Returns (success, content_or_error).
    """
    try:
        with open(path, 'r', encoding='utf-8') as f:
            return (True, f.read())
    except FileNotFoundError:
        return (False, "File not found")
    except PermissionError:
        return (False, "Permission denied")
    except UnicodeDecodeError:
        return (False, "Not a text file (binary content)")
    except IsADirectoryError:
        return (False, "Path is a directory")
    except Exception as e:
        return (False, f"Error: {type(e).__name__}")


def check_circular_reference(extracted_paths, output_path, base_dir):
    """
    Check if output file is among extracted paths.
    Returns True if circular reference detected.
    """
    output_resolved = output_path.resolve()
    for path_str in extracted_paths:
        resolved = resolve_path(path_str, base_dir)
        if resolved == output_resolved:
            return True
    return False


def build_results_log(successful, failed):
    """
    Build the results log to prepend to input file.
    """
    lines = [
        EXECUTION_LOG_MARKER,
        f"Execution timestamp: {time.strftime('%Y-%m-%d %H:%M:%S')}",
        "",
        f"Successfully concatenated ({len(successful)} files):",
    ]
    
    for path in sorted(successful):
        lines.append(f"  ✓ {path}")
    
    if failed:
        lines.append("")
        lines.append(f"Failed to read ({len(failed)} files):")
        for path, reason in sorted(failed):
            lines.append(f"  ✗ {path} — {reason}")
    
    lines.append("=" * 80)
    lines.append("")
    
    return "\n".join(lines)


def build_directory_context(base_dir):
    """
    Build directory context header for output file.
    """
    lines = [
        "CONCATENATED FILE BUNDLE",
        "Multiple source files merged for LLM context.",
        "Each file marked with '=== FILE: /absolute/path ==='",
        "",
        f"${base_dir}> pwd",
        f"{base_dir}",
        f"${base_dir}> ls -1",
    ]
    
    try:
        items = sorted(os.listdir(base_dir))
        for item in items:
            lines.append(item)
    except Exception as e:
        lines.append(f"Error listing directory: {e}")
    
    lines.append(f"${base_dir}> ")
    lines.append("")
    
    return "\n".join(lines)


def delete_ds_store_files(base_dir):
    """
    Recursively find and delete all .DS_Store files in base_dir.
    Returns count of deleted files.
    """
    deleted_count = 0
    for root, dirs, files in os.walk(base_dir):
        for filename in files:
            if filename == '.DS_Store':
                filepath = Path(root) / filename
                try:
                    filepath.unlink()
                    deleted_count += 1
                except Exception as e:
                    print(f"Warning: Failed to delete {filepath}: {e}")
    return deleted_count


def process_files(base_dir):
    """
    Main processing logic: read input, extract paths, concatenate, write output.
    """
    ds_store_count = delete_ds_store_files(base_dir)
    if ds_store_count > 0:
        print(f"Deleted {ds_store_count} .DS_Store files")
    
    input_path = base_dir / INPUT_FILE
    output_path = base_dir / OUTPUT_FILE
    
    # Read input file
    if not input_path.exists():
        print(f"Error: Input file '{INPUT_FILE}' not found")
        return
    
    try:
        with open(input_path, 'r', encoding='utf-8') as f:
            input_content = f.read()
    except Exception as e:
        print(f"Error reading input file: {e}")
        return
    
    # Check if this is a results log - skip processing if so
    first_line = input_content.split('\n')[0] if input_content else ""
    if EXECUTION_LOG_MARKER in first_line:
        # print("Skipping execution - results log detected in input file")
        return
    
    # Extract file paths
    extracted_paths = extract_file_paths(input_content)
    
    if not extracted_paths:
        print("No file paths found in input file")
        return
    
    # Deduplicate by resolved path while preserving order
    seen_resolved = set()
    unique_paths = []
    
    for path_str in extracted_paths:
        resolved = resolve_path(path_str, base_dir).resolve()
        if resolved not in seen_resolved:
            seen_resolved.add(resolved)
            unique_paths.append(path_str)
    
    print(f"Extracted {len(extracted_paths)} file paths ({len(unique_paths)} unique)")
    
    # Check for circular reference
    if check_circular_reference(unique_paths, output_path, base_dir):
        raise Exception(
            f"Circular reference detected: output file '{OUTPUT_FILE}' "
            "is among the extracted paths. Terminating to prevent infinite loop."
        )
    
    # Process each path - track both relative string and absolute for output
    successful = []
    failed = []
    concatenated_parts = []
    
    for path_str in unique_paths:
        resolved_path = resolve_path(path_str, base_dir)
        success, content_or_error = read_file_safe(resolved_path)
        
        absolute_path = str(resolved_path)
        
        if success:
            successful.append(absolute_path)
            concatenated_parts.append(f"=== FILE: {absolute_path} ===\n{content_or_error}\n")
        else:
            failed.append((absolute_path, content_or_error))
    
    # Write output file with directory context
    dir_context = build_directory_context(base_dir)
    file_contents = "\n".join(concatenated_parts)
    output_content = dir_context + file_contents
    
    try:
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(output_content)
        print(f"Wrote {len(successful)} files to '{OUTPUT_FILE}'")
    except Exception as e:
        print(f"Error writing output file: {e}")
        return
    
    # Copy to clipboard
    try:
        pyperclip.copy(output_content)
        print("Copied output to clipboard")
    except Exception as e:
        print(f"Warning: Failed to copy to clipboard: {e}")
    
    # Prepend results log to input file
    try:
        results_log = build_results_log(successful, failed)
        updated_input = results_log + input_content
        with open(input_path, 'w', encoding='utf-8') as f:
            f.write(updated_input)
        print("Updated input file with results log")
    except Exception as e:
        print(f"Warning: Failed to update input file with results: {e}")


# ============================================================================
# FILE WATCHING
# ============================================================================

class InputFileHandler(FileSystemEventHandler):
    def __init__(self, base_dir, input_file_path):
        self.base_dir = base_dir
        self.input_file_path = (base_dir / input_file_path).resolve()
        self.last_modified = 0
        
    def on_modified(self, event):
        if event.is_directory:
            return
        
        if Path(event.src_path).resolve() == self.input_file_path:
            # Debounce: ignore rapid successive changes
            current_time = time.time()
            if current_time - self.last_modified < FILE_CHANGE_DEBOUNCE:
                return
            self.last_modified = current_time
            
            print(f"\n[File change detected] Processing {INPUT_FILE}...")
            try:
                process_files(self.base_dir)
            except Exception as e:
                print(f"ERROR: {e}")


# ============================================================================
# CLIPBOARD WATCHING
# ============================================================================

def clipboard_watcher(base_dir):
    """
    Poll clipboard for activation signal.
    """
    last_clipboard = ""
    
    while True:
        time.sleep(CLIPBOARD_POLL_INTERVAL)
        
        try:
            current_clipboard = pyperclip.paste()
        except Exception:
            continue
        
        # Only trigger if clipboard changed AND matches activation signal
        if current_clipboard != last_clipboard:
            last_clipboard = current_clipboard
            
            if current_clipboard == CLIPBOARD_ACTIVATION_SIGNAL:
                print(f"\n[Clipboard activation detected] Processing...")
                try:
                    process_files(base_dir)
                except Exception as e:
                    print(f"ERROR: {e}")


# ============================================================================
# MAIN
# ============================================================================

def main():
    # Use current working directory where script is executed
    base_dir = Path.cwd()
    
    print("=" * 80)
    print("File Bundler Started")
    print("=" * 80)
    print(f"Working directory: {base_dir}")
    print(f"Input file: {INPUT_FILE}")
    print(f"Output file: {OUTPUT_FILE}")
    print(f"Clipboard activation signal: '{CLIPBOARD_ACTIVATION_SIGNAL}'")
    print("=" * 80)
    print("\nWatching for changes...")
    
    # Set up file watcher - watch the directory containing the input file
    event_handler = InputFileHandler(base_dir, INPUT_FILE)
    observer = Observer()
    watch_dir = (base_dir / INPUT_FILE).parent
    observer.schedule(event_handler, str(watch_dir), recursive=False)
    observer.start()
    
    # Run clipboard watcher (blocking)
    try:
        clipboard_watcher(base_dir)
    except KeyboardInterrupt:
        print("\n\nShutting down...")
        observer.stop()
    
    observer.join()


if __name__ == "__main__":
    main()