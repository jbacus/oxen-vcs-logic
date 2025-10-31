#!/bin/bash
# Extract ProjectData from all Logic Pro projects in the projects directory

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECTS_DIR="$SCRIPT_DIR/../projects"
BINARY_DIR="$SCRIPT_DIR/../binary_samples"
HEX_DIR="$SCRIPT_DIR/../hex_dumps"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo ""
echo "=========================================="
echo "Extract All Logic Pro Projects"
echo "=========================================="
echo ""

# Create output directories
mkdir -p "$BINARY_DIR"
mkdir -p "$HEX_DIR"

# Count projects
project_count=$(find "$PROJECTS_DIR" -name "*.logicx" -type d | wc -l | tr -d ' ')

if [ "$project_count" -eq 0 ]; then
    echo "No projects found in $PROJECTS_DIR"
    exit 1
fi

echo "Found $project_count projects"
echo ""

# Extract each project
count=0
for project in "$PROJECTS_DIR"/*.logicx; do
    if [ ! -d "$project" ]; then
        continue
    fi

    count=$((count + 1))
    project_name=$(basename "$project" .logicx)

    echo -e "${BLUE}[$count/$project_count] $project_name${NC}"

    # Find ProjectData file
    project_data=""
    if [ -f "$project/Alternatives/001/ProjectData" ]; then
        project_data="$project/Alternatives/001/ProjectData"
    elif [ -f "$project/Alternatives/000/ProjectData" ]; then
        project_data="$project/Alternatives/000/ProjectData"
    else
        echo -e "${YELLOW}  ⚠ ProjectData not found, skipping${NC}"
        continue
    fi

    # Extract binary
    cp "$project_data" "$BINARY_DIR/${project_name}.bin"

    # Generate hex dump
    hexdump -C "$BINARY_DIR/${project_name}.bin" > "$HEX_DIR/${project_name}.hex"

    # Get file size
    size=$(stat -f%z "$BINARY_DIR/${project_name}.bin")

    echo -e "${GREEN}  ✓ Extracted (${size} bytes)${NC}"
done

echo ""
echo "=========================================="
echo "Extraction Complete!"
echo "=========================================="
echo ""
echo "Extracted:"
echo "  - $count projects"
echo "  - Binary files: $BINARY_DIR/"
echo "  - Hex dumps: $HEX_DIR/"
echo ""
echo "Next steps:"
echo "  1. Compare projects: ./compare_pair.sh <name1> <name2>"
echo "  2. Analyze offsets: ./analyze_bytes.py analyze <file> <offset>"
echo ""
