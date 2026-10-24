# shft

`shft` is a fast, offline image converter and optimizer for your terminal. It is a single native Rust binary with safe output naming, deterministic presets, bounded parallelism, and no calls to network services or external media tools. It can also turn HEIC/HEIF photos into ordinary PNG, JPEG, or WebP files without requiring `libheif`, ImageMagick, or FFmpeg.

Turn a folder of oversized PNGs into web-ready images with:

```bash
shft ./assets --web
```

## Install

Install from [crates.io](https://crates.io/crates/shft) with Rust 1.88 or newer:

```bash
cargo install shft
```

Or build from source:

```bash
git clone https://github.com/aditipolkam/shft
cd shft
cargo install --path .
```

`cargo build --release` writes a local binary to `target/release/shft`.

## Quick start

```bash
# Convert formats
shft photo.png webp
shft photo.png --to webp
shft iphone-photo.heic jpg
shft transparent.heif png

# Re-encode without replacing the original
shft photo.jpg --compress
shft graphic.png --compress  # maximum lossless PNG compression effort

# Choose quality
shft photo.png webp -q 80

# Resize, always preserving aspect ratio
shft photo.jpg --width 1200
shft photo.jpg --resize 1200x800
shft photo.jpg --scale 50%
```

Unless `--overwrite` is present, `shft` never replaces an existing file. A same-format operation such as `shft photo.jpg --compress` creates `photo-optimized.jpg`. A conversion creates the natural extension (`photo.webp`). If an automatically derived destination already exists during an interactive single-file run, `shft` asks for a custom name. Batch, JSON, quiet, piped, and explicit `--output` runs report the conflict immediately and never wait for input. `--suffix`, `--output`, and `--output-dir` provide explicit naming control.

`--compress` uses quality 75 for JPEG and WebP unless `--quality` is supplied. Normal conversion defaults remain JPEG quality 85 and WebP quality 82. PNG compression remains lossless and uses maximum encoder effort, so its savings depend heavily on image content.

HEIC and HEIF are input-only formats. When no output format is given, an opaque HEIC/HEIF image becomes JPEG and one with alpha becomes PNG. Use `jpg`, `png`, `webp`, or `--to` to choose explicitly.

PNG is lossless and is usually a poor size choice for photographs. A compressed HEIC photo can expand many times when converted to PNG; `--compress` applies maximum lossless PNG compression but cannot recreate HEIC's lossy size reduction. Use JPEG or WebP for a small photographic result:

```bash
shft photo.heic jpg -q 85
shft photo.heic webp -q 82
```

## Web and other presets

```bash
shft image.png --web
shft ./images --web
shft ./images --web --recursive
shft avatar.png --preset avatar
```

`--web` is exactly equivalent to `--preset web`: WebP output, quality 82, a 2560-pixel maximum on both dimensions, no upscaling, and stripped ancillary metadata. Other deterministic presets are `thumbnail` (fit 400×400, WebP quality 75), `avatar` (fit 512×512, WebP quality 90), and `email` (JPEG for opaque images or WebP for transparent images, max width 1600, quality 72). Explicit format, quality, and resize flags override preset defaults.

## Smart mode

```bash
shft image.png --smart
shft image.png --smart --dry-run
```

Smart mode is entirely local and deterministic. It examines format, dimensions, file size, and actual alpha values. Large or opaque PNGs become WebP at quality 82 with a 2560-pixel cap; transparent assets use WebP at quality 90. Explicit CLI choices take priority.

## Target file size

```bash
shft photo.jpg --max-size 500kb
shft photo.webp --max-size 1mb --min-quality 30
```

For JPEG and WebP, `shft` performs at most eight quality probes and returns the highest quality found under the target. It never drops below quality 20 by default. If the target cannot be met, the smallest result is written with a warning. PNG is lossless, so a target that cannot be met reports a warning and suggests resizing or a lossy format.

## Batch processing

Directories are non-recursive by default. Add `-r` to recurse and `--jobs N` to bound the Rayon worker pool. Unsupported files are ignored. Individual failures do not stop a batch unless `--fail-fast` is used; any failed file makes the final exit status non-zero.

```bash
shft ./images --web --output-dir ./optimized
shft ./images --web -r --jobs 4
shft ./images --web --dry-run
```

Dry runs decode and validate source images and print exact output paths, but write nothing and do not claim estimated savings.

## Scripts and inspection

```bash
shft ./images --web --json
shft image.png --web --quiet
shft preview image.webp
```

JSON mode disables colors and progress output and includes per-file results and errors. Quiet mode emits only errors. `preview` reports format, dimensions, size, transparency, and EXIF orientation without changing the file.

## Transparency and metadata

PNG and WebP preserve alpha. JPEG does not: converting a genuinely transparent image to JPEG fails unless a background is supplied, for example `--background '#ffffff'`. EXIF orientation is applied to pixels before transforms so stripping metadata cannot rotate the visible result.

The current encoders write clean image payloads and therefore discard ancillary metadata. `--strip-metadata` makes that policy explicit and is enabled by optimization presets. `--keep-metadata` is accepted for forward compatibility, but metadata preservation is limited by the built-in codec APIs in the current release.

## Supported formats and limitations

- Decode: PNG, JPEG, WebP, HEIC, HEIF
- Encode: PNG, JPEG, WebP
- HEIC/HEIF decoding is pure Rust and supports common iPhone still images, grids, 8/10-bit HEVC, Display P3 conversion, orientation transforms, and supported alpha planes.
- HEIF image sequences, protected images, overlays, multi-layer images, 4:4:4 HEVC, and HEIF containers carrying AV1 or JPEG payloads are rejected cleanly.
- Deferred: AVIF, HEIC/HEIF encoding, crop modes, inline terminal rendering
- Animation is flattened to the decoder's first/default frame.
- Accepts one directory or multiple shell-expanded image paths per invocation; mix directories and files carefully because each directory keeps its own relative output tree.
- Image pixels are processed in memory one image per worker, so reduce `--jobs` for exceptionally large sources.

HEVC may be covered by patents in some jurisdictions. The bundled decoder is permissively licensed, but its copyright license does not grant third-party patent rights; distributors should evaluate their own requirements.

Run `shft --help` for all options.

## Contributing

Bug reports, feature requests and pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup and the checks CI runs, and [SECURITY.md](SECURITY.md) for reporting vulnerabilities privately.

## License

Licensed under the [MIT license](LICENSE-MIT).
