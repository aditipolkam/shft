pub mod options;
pub mod processor;

use std::{
    error::Error,
    io::IsTerminal,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::{
    config::RunConfig,
    error::{Result, ShftError},
    input::discovery,
    reporting::{FileFailure, Report},
};

pub fn run(config: &RunConfig) -> Result<Report> {
    let started = Instant::now();
    let mut files = Vec::new();
    for input in &config.inputs {
        files.extend(
            discovery::discover(input, config.recursive)?
                .into_iter()
                .map(|path| (path, input.clone())),
        );
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files.dedup_by(|a, b| a.0 == b.0);
    let interactive = std::io::stdout().is_terminal()
        && std::io::stderr().is_terminal()
        && !config.json
        && !config.quiet;
    let progress = if interactive && files.len() > 1 {
        let bar = ProgressBar::new(files.len() as u64);
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} [{bar:32.cyan/blue}] {pos}/{len} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_bar()),
        );
        Some(bar)
    } else {
        None
    };
    if interactive && files.len() > 1 {
        eprintln!("Processing {} images...", files.len());
    }
    let stopped = Arc::new(AtomicBool::new(false));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.jobs)
        .build()
        .map_err(|error| ShftError::ThreadPool(error.to_string()))?;
    let results = pool.install(|| {
        files
            .par_iter()
            .filter_map(|(path, input_root)| {
                if config.fail_fast && stopped.load(Ordering::Relaxed) {
                    return None;
                }
                if let Some(bar) = &progress {
                    bar.set_message(
                        path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                    );
                }
                let result = processor::process(path, input_root, config);
                if result.is_err() {
                    stopped.store(true, Ordering::Relaxed);
                }
                if let Some(bar) = &progress {
                    bar.inc(1);
                }
                Some((path.clone(), result))
            })
            .collect::<Vec<_>>()
    });
    if let Some(bar) = progress {
        bar.finish_and_clear();
    }
    let mut successes = Vec::new();
    let mut errors = Vec::new();
    for (path, result) in results {
        match result {
            Ok(file) => successes.push(file),
            Err(error) => errors.push(FileFailure {
                input: path,
                error: error_with_causes(&error),
            }),
        }
    }
    successes.sort_by(|a, b| a.input.cmp(&b.input));
    errors.sort_by(|a, b| a.input.cmp(&b.input));
    Ok(Report::new(
        successes,
        errors,
        started.elapsed(),
        config.dry_run,
    ))
}

fn error_with_causes(error: &ShftError) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str("; caused by: ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}
