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
    input: String,
    output: String,
    renderer_params: Option<String>,
    convert: Option<Option<String>>,
    style: Option<Option<String>>,
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut input = None;
    let mut output = None;
    let mut renderer_params = None;
    let mut convert: Option<Option<String>> = None;
    let mut style: Option<Option<String>> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" => {
                i += 1;
                input = args.get(i).cloned();
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
                if input.is_none() {
                    input = Some(args[i].clone());
                } else if output.is_none() {
                    output = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    let Some(input) = input else {
        eprintln!("Usage: app -i <input> -o <output.png> [-r [params]] [-c [params]] [-s [params]]");
        eprintln!("       app <input> <output.png>");
        eprintln!();
        eprintln!("  -i  Input file (SVG or PNG)");
        eprintln!("  -o  Output image file");
        eprintln!("  -r  Renderer params");
        eprintln!("  -c  Vectorize params (PNG is always vectorized; use -c to customize)");
        eprintln!("  -s  Style with VRC icon template & params");
        process::exit(1);
    };

    let Some(output) = output else {
        eprintln!("Error: output path is required (-o <path> or second positional arg)");
        process::exit(1);
    };

    Args {
        input,
        output,
        renderer_params,
        convert,
        style,
    }
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

fn main() {
    let args = parse_args();

    let input_path = Path::new(&args.input);
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let svg_data = match ext.as_str() {
        "png" => {
            let sp = spinner("Reading PNG...");
            let png_data = fs::read(&args.input).unwrap_or_else(|e| {
                sp.abandon();
                eprintln!("Failed to read {}: {e}", args.input);
                process::exit(1);
            });
            finish(&sp, "Read PNG");

            match &args.convert {
                Some(params) => {
                    let config = match params {
                        Some(s) => VtracerConfig::from_params_str(s).unwrap_or_else(|e| {
                            eprintln!("Invalid -c params: {e}");
                            process::exit(1);
                        }),
                        None => VtracerConfig::default(),
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
                None => {
                    let config = VtracerConfig::default();
                    let sp = spinner("Vectorizing...");
                    let result = vectorize(&png_data, &config).unwrap_or_else(|e| {
                        sp.abandon();
                        eprintln!("Vectorize failed: {e}");
                        process::exit(1);
                    });
                    finish(&sp, "Vectorized");
                    result
                }
            }
        }
        _ => {
            let sp = spinner("Reading SVG...");
            let result = fs::read_to_string(&args.input).unwrap_or_else(|e| {
                sp.abandon();
                eprintln!("Failed to read {}: {e}", args.input);
                process::exit(1);
            });
            finish(&sp, "Read SVG");
            result
        }
    };

    let svg_data = if args.style.is_some() {
        let sp = spinner("Normalizing SVG...");
        let result =
            normalize(&svg_data, &NormalizerConfig::default()).unwrap_or_else(|e| {
                sp.abandon();
                eprintln!("Normalize failed: {e}");
                process::exit(1);
            });
        finish(&sp, "Normalized SVG");
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
            finish(&sp, "Composed style");
            result
        }
        None => svg_data,
    };

    let output_path = Path::new(&args.output);
    let output_ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if output_ext == "svg" {
        let sp = spinner("Writing SVG...");
        fs::write(&args.output, &svg_data).unwrap_or_else(|e| {
            sp.abandon();
            eprintln!("Failed to write {}: {e}", args.output);
            process::exit(1);
        });
        finish(&sp, &format!("Saved {} -> {}", args.input, args.output));
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
                fs::write(&args.output, &png_bytes).unwrap_or_else(|e| {
                    sp.abandon();
                    eprintln!("Failed to write {}: {e}", args.output);
                    process::exit(1);
                });
                let w = options.width.map_or("auto".to_string(), |v| v.to_string());
                let h = options.height.map_or("auto".to_string(), |v| v.to_string());
                finish(
                    &sp,
                    &format!("Rendered {} -> {} ({w}x{h})", args.input, args.output),
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
