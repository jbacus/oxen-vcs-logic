#!/bin/bash
# Batch create Logic Pro test projects from a template
# This is more reliable than full AppleScript automation

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECTS_DIR="$SCRIPT_DIR/../projects"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo ""
echo "=========================================="
echo "Logic Pro Test Project Batch Creator"
echo "=========================================="
echo ""

# Check if template exists
if [ ! -d "$PROJECTS_DIR/template_120.logicx" ]; then
    echo -e "${RED}Error: Template project not found!${NC}"
    echo ""
    echo "Please create a template project first:"
    echo "  1. Open Logic Pro"
    echo "  2. Create a new empty project"
    echo "  3. Set tempo to 120 BPM"
    echo "  4. Add 1 software instrument track"
    echo "  5. Save as: $PROJECTS_DIR/template_120.logicx"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Template project found${NC}"
echo ""

# Function to duplicate and rename project
duplicate_project() {
    local new_name=$1
    local source="${PROJECTS_DIR}/template_120.logicx"
    local dest="${PROJECTS_DIR}/${new_name}.logicx"

    if [ -d "$dest" ]; then
        echo -e "${YELLOW}  ⚠ ${new_name}.logicx already exists, skipping${NC}"
        return
    fi

    cp -R "$source" "$dest"
    echo -e "${GREEN}  ✓ Created ${new_name}.logicx${NC}"
}

# Create tempo variations
echo "Creating tempo test projects..."
echo "================================"

TEMPO_VALUES=(60 90 128 140 180)

for tempo in "${TEMPO_VALUES[@]}"; do
    duplicate_project "tempo_${tempo}"
done

echo ""
echo -e "${YELLOW}⚠ IMPORTANT: You must manually set the tempo in each project!${NC}"
echo ""
echo "To set tempo in each project:"
echo "  1. Open the project in Logic Pro"
echo "  2. Click the tempo display (top center)"
echo "  3. Type the new tempo value"
echo "  4. Press Enter"
echo "  5. Save (Cmd+S) and close (Cmd+W)"
echo ""
echo "Projects to update:"
for tempo in "${TEMPO_VALUES[@]}"; do
    if [ "$tempo" -ne 120 ]; then
        echo "  - tempo_${tempo}.logicx → Set tempo to ${tempo} BPM"
    fi
done

echo ""
echo "Would you like to open them one by one? (y/n)"
read -r response

if [[ "$response" =~ ^[Yy]$ ]]; then
    for tempo in "${TEMPO_VALUES[@]}"; do
        if [ "$tempo" -ne 120 ]; then
            echo ""
            echo "=========================================="
            echo "Opening: tempo_${tempo}.logicx"
            echo "Set tempo to: ${tempo} BPM"
            echo "Then save and close"
            echo "=========================================="
            echo "Press Enter to open..."
            read

            open "${PROJECTS_DIR}/tempo_${tempo}.logicx"

            echo "Waiting for you to update and close the project..."
            echo "Press Enter when done..."
            read
        fi
    done

    echo ""
    echo -e "${GREEN}✓ All tempo projects created and updated!${NC}"
fi

# Create sample rate variations
echo ""
echo "Creating sample rate test projects..."
echo "====================================="

SR_VALUES=(44100 48000 96000)

for sr in "${SR_VALUES[@]}"; do
    duplicate_project "sr_${sr}"
done

echo ""
echo -e "${YELLOW}⚠ Note: Sample rate must be set in project settings${NC}"
echo "These will require manual adjustment in Logic Pro preferences"

# Create key signature variations
echo ""
echo "Creating key signature test projects..."
echo "========================================"

KEYS=("c_major" "d_major" "a_minor")
for key in "${KEYS[@]}"; do
    duplicate_project "key_${key}"
done

echo ""
echo -e "${GREEN}✓ Batch creation complete!${NC}"
echo ""
echo "Summary:"
echo "  - $(ls -1d ${PROJECTS_DIR}/*.logicx 2>/dev/null | wc -l) projects in ${PROJECTS_DIR}/"
echo ""
echo "Next steps:"
echo "  1. Manually set parameters in each project"
echo "  2. Run: cd ../scripts && ./extract_all.sh"
echo "  3. Start comparing projects"
echo ""
