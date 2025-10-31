#!/bin/bash
# Extract ProjectData binary from a Logic Pro project

set -e

if [ $# -lt 2 ]; then
    echo "Usage: $0 <project_name.logicx> <output_name>"
    echo "Example: $0 tempo_120.logicx tempo_120"
    exit 1
fi

PROJECT=$1
OUTPUT=$2

# Check if project exists
if [ ! -d "$PROJECT" ]; then
    echo "Error: Project directory not found: $PROJECT"
    exit 1
fi

# Find ProjectData file
PROJECT_DATA=""
if [ -f "$PROJECT/Alternatives/001/ProjectData" ]; then
    PROJECT_DATA="$PROJECT/Alternatives/001/ProjectData"
elif [ -f "$PROJECT/Alternatives/000/ProjectData" ]; then
    PROJECT_DATA="$PROJECT/Alternatives/000/ProjectData"
else
    echo "Error: ProjectData not found in $PROJECT"
    exit 1
fi

# Create output directory
mkdir -p ../binary_samples
mkdir -p ../hex_dumps

# Extract binary
cp "$PROJECT_DATA" "../binary_samples/${OUTPUT}.bin"
echo "✓ Extracted binary: binary_samples/${OUTPUT}.bin"

# Generate hex dump
hexdump -C "../binary_samples/${OUTPUT}.bin" > "../hex_dumps/${OUTPUT}.hex"
echo "✓ Created hex dump: hex_dumps/${OUTPUT}.hex"

# Show file size
SIZE=$(stat -f%z "../binary_samples/${OUTPUT}.bin")
echo "✓ File size: $SIZE bytes"
