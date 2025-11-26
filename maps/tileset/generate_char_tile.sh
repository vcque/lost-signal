#!/bin/bash

# Generate PNG tile from character and color
# Usage: ./generate_char_tile.sh <char> <color> [output_name]
# Example: ./generate_char_tile.sh '@' '#ff0000' player.png

set -e

if [ $# -lt 2 ]; then
    echo "Usage: $0 <char> <color> [output_name]"
    echo "  char: Single character to render"
    echo "  color: Color in #rrggbb format"
    echo "  output_name: Optional output filename (default: char_RRGGBB.png)"
    exit 1
fi

CHAR="$1"
COLOR="$2"
OUTPUT_NAME="$3"

# Validate color format
if [[ ! $COLOR =~ ^#[0-9a-fA-F]{6}$ ]]; then
    echo "Error: Color must be in #rrggbb format (e.g., #ff0000)"
    exit 1
fi

# Generate default filename if not provided
if [ -z "$OUTPUT_NAME" ]; then
    COLOR_CODE=$(echo "$COLOR" | tr '[:lower:]' '[:upper:]' | sed 's/#//')
    OUTPUT_NAME="char_${COLOR_CODE}.png"
fi

# Ensure we're in the right directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Create temporary SVG file
SVG_FILE="/tmp/char_tile_$$.svg"
PNG_FILE="$OUTPUT_NAME"

# Generate SVG with the character
cat > "$SVG_FILE" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg width="16" height="32" viewBox="0 0 16 32" xmlns="http://www.w3.org/2000/svg">
  <rect width="16" height="32" fill="black"/>
  <text x="8" y="25" font-family="monospace" font-size="32" fill="$COLOR" text-anchor="middle">$CHAR</text>
</svg>
EOF

# Convert SVG to PNG using ImageMagick (if available) or Inkscape
if command -v convert >/dev/null 2>&1; then
    magick "$SVG_FILE" "$PNG_FILE"
elif command -v inkscape >/dev/null 2>&1; then
    inkscape --export-type=png --export-filename="$PNG_FILE" --export-width=16 --export-height=32 "$SVG_FILE" >/dev/null 2>&1
elif command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w 16 -h 32 -o "$PNG_FILE" "$SVG_FILE"
else
    echo "Error: No suitable SVG converter found. Please install one of:"
    echo "  - ImageMagick (convert command)"
    echo "  - Inkscape"
    echo "  - librsvg (rsvg-convert command)"
    rm "$SVG_FILE"
    exit 1
fi

# Clean up temporary file
rm "$SVG_FILE"

echo "Generated: $PNG_FILE (character '$CHAR' in color $COLOR)"
