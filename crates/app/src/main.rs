use compositor::{CompositorConfig, compose};
use converter::{VtracerConfig, vectorize};
use indicatif::{ProgressBar, ProgressStyle};
use normalizer::{NormalizerConfig, normalize};
use renderer::{RenderOptions, render_svg};
use std::env;
use std::fs;
use std::path::Path;
use std::process;

struct Args {
    inputs: Vec<String>,
    output: String,
    renderer_params: Option<String>,
    convert: Option<Option<String>>,
    style: Option<Option<String>>,
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut inputs = Vec::new();
    let mut output = None;
    let mut renderer_params = None;
    let mut convert: Option<Option<String>> = None;
    let mut style: Option<Option<String>> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    inputs.push(v.clone());
                }
            }
            "-o" => {
                i += 1;
                output = args.get(i).cloned();
            }
            "-r" => {
                i += 1;
                renderer_params = args.get(i).cloned();
            }
            "-c" => {
                if let Some(next) = args.get(i + 1) {
                    if !next.starts_with('-') {
                        convert = Some(Some(next.clone()));
                        i += 1;
                    } else {
                        convert = Some(None);
                    }
                } else {
                    convert = Some(None);
                }
            }
            "-s" => {
                if let Some(next) = args.get(i + 1) {
                    if !next.starts_with('-') {
                        style = Some(Some(next.clone()));
                        i += 1;
                    } else {
                        style = Some(None);
                    }
                } else {
                    style = Some(None);
                }
            }
            _ => {
                if output.is_some() || !inputs.is_empty() {
                    if output.is_none() {
                        output = Some(args[i].clone());
                    }
                } else {
                    inputs.push(args[i].clone());
                }
            }
        }
        i += 1;
    }

    if inputs.is_empty() {
        eprintln!("Usage: app -i <input> -o <output> [-r [params]] [-c [params]] [-s [params]]");
        eprintln!("       app <input> <output>");
        eprintln!("       app -i <glob_pattern> -o <output_dir>");
        eprintln!();
        eprintln!("  -i  Input file, glob pattern, or directory (can specify multiple)");
        eprintln!("  -o  Output file or directory");
        eprintln!("  -r  Renderer params");
        eprintln!("  -c  Vectorize params (PNG is always vectorized; use -c to customize)");
        eprintln!("  -s  Style with VRC icon template & params");
        process::exit(1);
    }

    let Some(output) = output else {
        eprintln!("Error: output path is required (-o <path> or second positional arg)");
        process::exit(1);
    };

    Args {
        inputs,
        output,
        renderer_params,
        convert,
        style,
    }
}

fn expand_inputs(raw: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for pattern in raw {
        let path = Path::new(pattern);
        if path.is_dir() {
            for ext in &["*.svg", "*.png", "*.SVG", "*.PNG"] {
                let glob_pattern = format!("{}/{ext}", pattern.trim_end_matches(['/', '\\']));
                if let Ok(paths) = glob::glob(&glob_pattern) {
                    for entry in paths.flatten() {
                        result.push(entry.display().to_string());
                    }
                }
            }
        } else if pattern.contains('*') || pattern.contains('?') {
            if let Ok(paths) = glob::glob(pattern) {
                for entry in paths.flatten() {
                    result.push(entry.display().to_string());
                }
            }
        } else {
            result.push(pattern.clone());
        }
    }
    result
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg} [{elapsed}]")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb.set_message(msg.to_string());
    pb
}

fn finish(pb: &ProgressBar, msg: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("  {msg} [{elapsed}]")
            .unwrap(),
    );
    pb.finish_with_message(msg.to_string());
}

fn process_file(
    input_path: &str,
    output_path: &str,
    args: &Args,
) {
    let path = Path::new(input_path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let svg_data = match ext.as_str() {
        "png" => {
            let sp = spinner(&format!("Reading {input_path}..."));
            let png_data = fs::read(input_path).unwrap_or_else(|e| {
                sp.abandon();
                eprintln!("Failed to read {input_path}: {e}");
                process::exit(1);
            });
            finish(&sp, &format!("Read {input_path}"));

            let config = match &args.convert {
                Some(Some(s)) => VtracerConfig::from_params_str(s).unwrap_or_else(|e| {
                    eprintln!("Invalid -c params: {e}");
                    process::exit(1);
                }),
                _ => VtracerConfig::default(),
            };
            let sp = spinner("Vectorizing...");
            let result = vectorize(&png_data, &config).unwrap_or_else(|e| {
                sp.abandon();
                eprintln!("Vectorize failed: {e}");
                process::exit(1);
            });
            finish(&sp, "Vectorized");
            result
        }
        _ => {
            let sp = spinner(&format!("Reading {input_path}..."));
            let result = fs::read_to_string(input_path).unwrap_or_else(|e| {
                sp.abandon();
                eprintln!("Failed to read {input_path}: {e}");
                process::exit(1);
            });
            finish(&sp, &format!("Read {input_path}"));
            result
        }
    };

    let svg_data = if args.style.is_some() {
        let sp = spinner("Normalizing...");
        let result =
            normalize(&svg_data, &NormalizerConfig::default()).unwrap_or_else(|e| {
                sp.abandon();
                eprintln!("Normalize failed: {e}");
                process::exit(1);
            });
        finish(&sp, "Normalized");
        result
    } else {
        svg_data
    };

    let svg_data = match &args.style {
        Some(params) => {
            let config = match params {
                Some(s) => CompositorConfig::from_params_str(s).unwrap_or_else(|e| {
                    eprintln!("Invalid -s params: {e}");
                    process::exit(1);
                }),
                None => CompositorConfig::default(),
            };
            let sp = spinner("Composing style...");
            let result = compose(&svg_data, &config).unwrap_or_else(|e| {
                sp.abandon();
                eprintln!("Compose failed: {e}");
                process::exit(1);
            });
            finish(&sp, "Composed");
            result
        }
        None => svg_data,
    };

    let output_ext = Path::new(output_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if output_ext == "svg" {
        let sp = spinner("Writing SVG...");
        fs::write(output_path, &svg_data).unwrap_or_else(|e| {
            sp.abandon();
            eprintln!("Failed to write {output_path}: {e}");
            process::exit(1);
        });
        finish(&sp, &format!("Saved {input_path} -> {output_path}"));
    } else {
        let options = match &args.renderer_params {
            Some(s) => RenderOptions::from_params_str(s).unwrap_or_else(|e| {
                eprintln!("Invalid -r params: {e}");
                process::exit(1);
            }),
            None => RenderOptions::default(),
        };

        let sp = spinner("Rendering PNG...");
        match render_svg(&svg_data, &options) {
            Ok(png_bytes) => {
                sp.set_message("Writing PNG...".to_string());
                fs::write(output_path, &png_bytes).unwrap_or_else(|e| {
                    sp.abandon();
                    eprintln!("Failed to write {output_path}: {e}");
                    process::exit(1);
                });
                let w = options.width.map_or("auto".to_string(), |v| v.to_string());
                let h = options.height.map_or("auto".to_string(), |v| v.to_string());
                finish(
                    &sp,
                    &format!("Rendered {input_path} -> {output_path} ({w}x{h})"),
                );
            }
            Err(e) => {
                sp.abandon();
                eprintln!("Render failed: {e}");
                process::exit(1);
            }
        }
    }
}

fn main() {
    let args = parse_args();
    let inputs = expand_inputs(&args.inputs);

    if inputs.is_empty() {
        eprintln!("No input files matched");
        process::exit(1);
    }

    let output_is_dir = Path::new(&args.output).is_dir()
        || (inputs.len() > 1 && !args.output.contains('.'));

    if inputs.len() > 1 && output_is_dir {
        let out_dir = Path::new(&args.output);
        if !out_dir.exists() {
            fs::create_dir_all(out_dir).unwrap_or_else(|e| {
                eprintln!("Failed to create output directory {}: {e}", args.output);
                process::exit(1);
            });
        }
    }

    if inputs.len() > 1 {
        eprintln!("Processing {} files...", inputs.len());
    }

    for input in &inputs {
        let output_path = if output_is_dir {
            let stem = Path::new(input)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            format!("{}/{stem}.png", args.output.trim_end_matches(['/', '\\']))
        } else {
            args.output.clone()
        };

        process_file(input, &output_path, &args);
    }
}
