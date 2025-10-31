#!/bin/bash
# Automated workflow to create and configure tempo test projects
# This combines project duplication with automated tempo setting

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECTS_DIR="$SCRIPT_DIR/../projects"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo ""
echo "=========================================="
echo "Automated Tempo Test Project Workflow"
echo "=========================================="
echo ""

# Check for template
if [ ! -d "$PROJECTS_DIR/template_120.logicx" ]; then
    echo "Creating template project..."
    echo ""
    echo "Please follow these steps:"
    echo "  1. Open Logic Pro"
    echo "  2. File > New"
    echo "  3. Choose 'Empty Project'"
    echo "  4. Add 1 Software Instrument track (or skip)"
    echo "  5. Set tempo to 120 BPM"
    echo "  6. File > Save As: template_120"
    echo "  7. Save location: $PROJECTS_DIR"
    echo "  8. Close the project"
    echo ""
    echo "Press Enter when done..."
    read

    if [ ! -d "$PROJECTS_DIR/template_120.logicx" ]; then
        echo "Error: Template not found. Exiting."
        exit 1
    fi
fi

echo -e "${GREEN}✓ Template found${NC}"
echo ""

# Define tempo test cases
declare -A TEMPO_PROJECTS=(
    ["tempo_60"]=60
    ["tempo_90"]=90
    ["tempo_120"]=120
    ["tempo_128"]=128
    ["tempo_140"]=140
    ["tempo_180"]=180
)

# Create/update each project
for project_name in "${!TEMPO_PROJECTS[@]}"; do
    tempo_value="${TEMPO_PROJECTS[$project_name]}"
    project_path="$PROJECTS_DIR/${project_name}.logicx"

    echo "=========================================="
    echo "Processing: $project_name (${tempo_value} BPM)"
    echo "=========================================="

    # Duplicate template if doesn't exist
    if [ ! -d "$project_path" ]; then
        echo -e "${BLUE}Creating $project_name...${NC}"
        cp -R "$PROJECTS_DIR/template_120.logicx" "$project_path"
        echo -e "${GREEN}✓ Created${NC}"
    else
        echo -e "${YELLOW}Already exists${NC}"
    fi

    # Skip if already at correct tempo
    if [ "$tempo_value" -eq 120 ]; then
        echo "Tempo already correct (120 BPM)"
        echo ""
        continue
    fi

    echo "Opening project and setting tempo to ${tempo_value}..."

    # Open project
    open "$project_path"
    sleep 5  # Wait for Logic Pro to open

    # Set tempo using AppleScript
    osascript "$SCRIPT_DIR/set_tempo_in_project.applescript" "$tempo_value"
    sleep 2

    # Save and close
    osascript <<EOF
tell application "Logic Pro"
    activate
    tell application "System Events"
        tell process "Logic Pro"
            -- Save
            keystroke "s" using command down
            delay 1
            -- Close
            keystroke "w" using command down
            delay 1
        end tell
    end tell
end tell
EOF

    echo -e "${GREEN}✓ Updated tempo to ${tempo_value} BPM${NC}"
    echo ""
    sleep 2
done

echo ""
echo "=========================================="
echo "Workflow Complete!"
echo "=========================================="
echo ""
echo "Created projects:"
for project_name in "${!TEMPO_PROJECTS[@]}"; do
    tempo_value="${TEMPO_PROJECTS[$project_name]}"
    echo "  - $project_name.logicx (${tempo_value} BPM)"
done

echo ""
echo "Next steps:"
echo "  1. Verify projects in Logic Pro (optional)"
echo "  2. Extract binaries: ./extract_all.sh"
echo "  3. Compare projects: ./compare_pair.sh tempo_120 tempo_128"
echo ""
