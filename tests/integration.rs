use std::fs;

use assert_cmd::Command;
use image::{ImageBuffer, Rgba};

fn png(path: &std::path::Path, alpha: u8) {
    let image = ImageBuffer::from_pixel(8, 6, Rgba([200_u8, 100, 50, alpha]));
    image.save(path).unwrap();
}

#[test]
fn dry_run_creates_no_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("sample.png");
    png(&input, 255);
    Command::cargo_bin("shft")
        .unwrap()
        .args([input.to_str().unwrap(), "webp", "--dry-run"])
        .assert()
        .success();
    assert!(!dir.path().join("sample.webp").exists());
}

#[test]
fn converts_png_to_webp() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("sample.png");
    png(&input, 255);
    Command::cargo_bin("shft")
        .unwrap()
        .args([input.to_str().unwrap(), "webp"])
        .assert()
        .success();
    assert!(dir.path().join("sample.webp").exists());
}

#[test]
fn transparent_png_to_jpeg_requires_background() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("alpha.png");
    png(&input, 100);
    Command::cargo_bin("shft")
        .unwrap()
        .args([input.to_str().unwrap(), "jpg"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("transparent image"));
    assert!(!dir.path().join("alpha.jpg").exists());
}

#[test]
fn existing_output_is_protected() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("sample.png");
    png(&input, 255);
    let output = dir.path().join("sample.webp");
    fs::write(&output, b"keep").unwrap();
    Command::cargo_bin("shft")
        .unwrap()
        .args([input.to_str().unwrap(), "webp"])
        .assert()
        .failure();
    assert_eq!(fs::read(output).unwrap(), b"keep");
}

#[test]
fn malformed_heic_is_recognized_and_rejected_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("broken.heic");
    fs::write(&input, b"not a heif container").unwrap();
    Command::cargo_bin("shft")
        .unwrap()
        .args([input.to_str().unwrap(), "jpg"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("failed to decode"));
}
