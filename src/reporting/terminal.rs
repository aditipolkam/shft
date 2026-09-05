use std::path::Path;

use console::Style;

use crate::reporting::Report;

pub fn print(report: &Report, colors: bool) {
    let success = if colors {
        Style::new().green()
    } else {
        Style::new()
    };
    let danger = if colors {
        Style::new().red()
    } else {
        Style::new()
    };
    if report.dry_run {
        println!(
            "Dry run: {} supported image{} found\n",
            report.processed,
            plural(report.processed)
        );
    }
    for file in &report.files {
        if report.dry_run {
            println!(
                "{} {} → {}",
                success.apply_to("•"),
                short(&file.input),
                file.output.display()
            );
            println!(
                "  Input: {} {}×{}, {}, transparency {}",
                crate::codecs::ImageFormat::from_path(&file.input)
                    .map_or_else(|| "unknown".to_string(), |format| format.to_string()),
                file.source_width,
                file.source_height,
                bytes(file.input_bytes),
                if file.has_alpha { "detected" } else { "none" }
            );
            println!(
                "  Plan: {} {}×{}, quality {}, metadata {}",
                file.format,
                file.width,
                file.height,
                file.quality
                    .map_or_else(|| "lossless".into(), |quality| quality.to_string()),
                if file.strip_metadata {
                    "strip"
                } else {
                    "preserve when supported"
                }
            );
        } else {
            println!(
                "{} {} → {}",
                success.apply_to("✓"),
                short(&file.input),
                file.output.display()
            );
            if let Some(output_bytes) = file.output_bytes {
                println!("  {} → {}", bytes(file.input_bytes), bytes(output_bytes));
            }
            if (file.source_width, file.source_height) != (file.width, file.height) {
                println!(
                    "  {}×{} → {}×{}",
                    file.source_width, file.source_height, file.width, file.height
                );
            }
            if let Some(quality) = file.quality {
                println!("  Quality {quality}");
            }
            println!("  {:.2}s", file.elapsed_ms as f64 / 1000.0);
        }
        if let Some(warning) = &file.warning {
            eprintln!("  warning: {warning}");
        }
    }
    for error in &report.errors {
        eprintln!(
            "{} {}: {}",
            danger.apply_to("✗"),
            short(&error.input),
            error.error
        );
    }
    if report.processed > 1 {
        println!(
            "\n{} processed\n{} succeeded\n{} failed",
            report.processed, report.succeeded, report.failed
        );
        if !report.dry_run && report.succeeded > 0 {
            println!(
                "\n{} → {}",
                bytes(report.input_bytes),
                bytes(report.output_bytes)
            );
            if report.input_bytes > report.output_bytes {
                let saved = report.input_bytes - report.output_bytes;
                println!(
                    "Saved {} ({:.0}%)",
                    bytes(saved),
                    saved as f64 / report.input_bytes as f64 * 100.0
                );
            }
        }
    }
    if report.dry_run {
        println!("\nEstimated size savings unavailable in dry-run mode");
    }
}

fn short(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}
pub fn bytes(value: u64) -> String {
    if value >= 1_000_000 {
        format!("{:.1} MB", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.0} KB", value as f64 / 1_000.0)
    } else {
        format!("{value} B")
    }
}
