#!/bin/bash
# Compare two ProjectData binaries and highlight differences

set -e

if [ $# -lt 2 ]; then
    echo "Usage: $0 <file1_name> <file2_name>"
    echo "Example: $0 tempo_120 tempo_128"
    echo ""
    echo "Files should exist in binary_samples/ directory"
    exit 1
fi

FILE1=$1
FILE2=$2

BIN1="../binary_samples/${FILE1}.bin"
BIN2="../binary_samples/${FILE2}.bin"
HEX1="../hex_dumps/${FILE1}.hex"
HEX2="../hex_dumps/${FILE2}.hex"

# Check files exist
if [ ! -f "$BIN1" ]; then
    echo "Error: $BIN1 not found"
    exit 1
fi

if [ ! -f "$BIN2" ]; then
    echo "Error: $BIN2 not found"
    exit 1
fi

echo "=========================================="
echo "Comparing: $FILE1 vs $FILE2"
echo "=========================================="
echo ""

# Show file sizes
SIZE1=$(stat -f%z "$BIN1")
SIZE2=$(stat -f%z "$BIN2")
echo "File sizes:"
echo "  $FILE1: $SIZE1 bytes"
echo "  $FILE2: $SIZE2 bytes"
echo ""

# Hex dump comparison
echo "Creating diff report..."
mkdir -p ../findings

DIFF_FILE="../findings/${FILE1}_vs_${FILE2}_diff.txt"
diff -u "$HEX1" "$HEX2" > "$DIFF_FILE" || true

# Count differences
DIFF_LINES=$(grep -c "^[+-]" "$DIFF_FILE" || echo "0")
echo "✓ Diff saved to: findings/${FILE1}_vs_${FILE2}_diff.txt"
echo "✓ Changed lines: $DIFF_LINES"
echo ""

# Show first few differences
echo "First 20 differences:"
echo "--------------------"
grep -A2 "^@@" "$DIFF_FILE" | head -60

echo ""
echo "Full diff available in: $DIFF_FILE"
