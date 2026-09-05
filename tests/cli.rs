use clap::Parser;
use shft::{cli::Cli, codecs::ImageFormat, config::RunConfig, presets::Preset};

#[test]
fn parses_positional_format() {
    let cli = Cli::try_parse_from(["shft", "photo.png", "jpeg", "-q", "80"]).unwrap();
    assert_eq!(cli.transform.format.as_deref(), Some("jpeg"));
    assert_eq!(cli.transform.quality, Some(80));
}

#[test]
fn rejects_invalid_quality() {
    assert!(Cli::try_parse_from(["shft", "photo.png", "-q", "200"]).is_err());
}

#[test]
fn explicit_flags_override_preset() {
    let cli = Cli::try_parse_from([
        "shft",
        "photo.png",
        "--preset",
        "thumbnail",
        "--to",
        "png",
        "-q",
        "95",
        "--max-width",
        "900",
    ])
    .unwrap();
    let config = RunConfig::resolve(cli.transform).unwrap();
    assert_eq!(config.preset, Some(Preset::Thumbnail));
    assert_eq!(config.explicit_format, Some(ImageFormat::Png));
    assert_eq!(config.quality, Some(95));
    assert_eq!(config.resize.max_width, Some(900));
}

#[test]
fn explicit_resize_replaces_preset_bounds() {
    let cli = Cli::try_parse_from([
        "shft",
        "photo.png",
        "--preset",
        "thumbnail",
        "--width",
        "900",
    ])
    .unwrap();
    let config = RunConfig::resolve(cli.transform).unwrap();
    assert_eq!(config.resize.width, Some(900));
    assert_eq!(config.resize.bounds, None);
}

#[test]
fn accepts_shell_expanded_inputs() {
    let cli = Cli::try_parse_from(["shft", "one.png", "two.jpg", "three.webp", "--web"]).unwrap();
    let config = RunConfig::resolve(cli.transform).unwrap();
    assert_eq!(config.inputs.len(), 3);
}

#[test]
fn rejects_heic_as_an_output_format() {
    let cli = Cli::try_parse_from(["shft", "photo.png", "--to", "heic"]).unwrap();
    let error = RunConfig::resolve(cli.transform).unwrap_err();
    assert!(error.to_string().contains("input-only"));
}
