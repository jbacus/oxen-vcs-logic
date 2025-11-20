# Auxin Icon Assets

This directory contains the icon assets for the Auxin application.

## Files

- **auxin-icon.pdf** - Original icon design (vector format)
- **Auxin.icns** - macOS icon bundle (used by the app)
- **Auxin.iconset/** - Icon set directory containing PNG files at multiple resolutions
- **icon-128.png** - 128x128 PNG for documentation and README files

## Icon Design

The icon features a botanical design (fruit/berry with leaves) representing:
- **Growth** - Fitting for version control tracking project evolution
- **Branching** - Natural representation of git-like branching
- **Organic development** - Reflecting creative workflows

The name "Auxin" refers to plant growth hormones, making the botanical theme appropriate.

## Where the Icon is Used

### macOS Application
- **Auxin.app** - App bundle icon via `CFBundleIconFile` in Info.plist
- **Auxin-App/Auxin.app/Contents/Resources/Auxin.icns** - Icon file location
- **Auxin-App/create-app-bundle.sh** - Automatically copies icon during build

### Web Interface (auxin-server)
- **Frontend favicon** - `favicon-32.png` (32x32)
- **PWA icons** - Multiple sizes (96, 192, 512) for progressive web app support
- **Header logo** - `icon-96.png` displayed in the web interface header
- **manifest.json** - PWA manifest with icon references
- Located in: `auxin-server/frontend/public/`

### Documentation
The icon appears at the top of:
- **README.md** (main project)
- **Auxin-CLI-Wrapper/README.md**
- **auxin-server/README.md**
- **docs/user/README.md**
- **docs/developer/README.md**

## Usage

The icon is automatically copied to the app bundle during build by `Auxin-App/create-app-bundle.sh`.

### Manual Integration

If you need to manually update the icon:

1. Edit `auxin-icon.pdf` with your design tool
2. Regenerate the icon files:
   ```bash
   cd assets/icon

   # Regenerate iconset
   sips -s format png auxin-icon.pdf --resampleWidth 1024 --out Auxin.iconset/icon_512x512@2x.png
   sips -s format png auxin-icon.pdf --resampleWidth 512 --out Auxin.iconset/icon_512x512.png
   sips -s format png auxin-icon.pdf --resampleWidth 512 --out Auxin.iconset/icon_256x256@2x.png
   sips -s format png auxin-icon.pdf --resampleWidth 256 --out Auxin.iconset/icon_256x256.png
   sips -s format png auxin-icon.pdf --resampleWidth 256 --out Auxin.iconset/icon_128x128@2x.png
   sips -s format png auxin-icon.pdf --resampleWidth 128 --out Auxin.iconset/icon_128x128.png
   sips -s format png auxin-icon.pdf --resampleWidth 64 --out Auxin.iconset/icon_32x32@2x.png
   sips -s format png auxin-icon.pdf --resampleWidth 32 --out Auxin.iconset/icon_32x32.png
   sips -s format png auxin-icon.pdf --resampleWidth 32 --out Auxin.iconset/icon_16x16@2x.png
   sips -s format png auxin-icon.pdf --resampleWidth 16 --out Auxin.iconset/icon_16x16.png

   # Generate .icns
   iconutil -c icns Auxin.iconset -o Auxin.icns

   # Regenerate web icons
   sips -s format png auxin-icon.pdf --resampleWidth 192 --out ../../auxin-server/frontend/public/icon-192.png
   sips -s format png auxin-icon.pdf --resampleWidth 512 --out ../../auxin-server/frontend/public/icon-512.png
   sips -s format png auxin-icon.pdf --resampleWidth 32 --out ../../auxin-server/frontend/public/favicon-32.png
   sips -s format png auxin-icon.pdf --resampleWidth 96 --out ../../auxin-server/frontend/public/icon-96.png

   # Regenerate documentation icon
   sips -s format png auxin-icon.pdf --resampleWidth 128 --out icon-128.png
   ```

3. Rebuild the app:
   ```bash
   cd ../../Auxin-App
   ./create-app-bundle.sh
   ```

4. Rebuild the web interface:
   ```bash
   cd ../../auxin-server/frontend
   npm run build
   ```

## Icon Specifications

The .icns file contains the following sizes:
- 1024x1024 (@2x for 512x512)
- 512x512
- 256x256 (@2x for 128x128)
- 128x128
- 64x64 (@2x for 32x32)
- 32x32 (@2x for 16x16)
- 16x16

These sizes ensure the icon looks sharp at all resolutions in Finder, Dock, and other macOS UI elements.
