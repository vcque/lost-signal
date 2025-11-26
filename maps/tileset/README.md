# Character Tile Generator

A bash script to generate 16x32 PNG tiles from characters with specified colors on black backgrounds.

## Usage

```bash
./generate_char_tile.sh <char> <color> [output_name]
```

### Parameters

- `char`: Single character to render (e.g., '@', '#', 'X')
- `color`: Color in #rrggbb format (e.g., #ff0000 for red)
- `output_name`: Optional output filename (default: char_RRGGBB.png)

### Examples

```bash
# Generate a red '@' symbol
./generate_char_tile.sh '@' '#ff0000'
# Output: char_FF0000.png

# Generate a blue wall character with custom name
./generate_char_tile.sh '#' '#0000ff' wall_blue.png
# Output: wall_blue.png

# Generate a green tree
./generate_char_tile.sh 'T' '#00ff00' tree.png
# Output: tree.png
```

## Output

The script generates 16x32 pixel PNG files with black backgrounds and the specified character rendered in monospace font in the given color.