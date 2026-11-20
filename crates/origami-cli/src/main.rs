//! FOLD validation CLI: structure + flat-foldability gates. Composes on
//! `origami-core`; the logic lives in `lib.rs` so it stays testable.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Validates crease patterns in the design library and beyond.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// What to do.
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check FOLD file(s): structural invariants + Maekawa + Kawasaki.
    FoldCheck {
        /// A .fold file, or a directory (checks every *.fold inside).
        path: PathBuf,
        /// Treat the path as a directory and check everything under it.
        #[arg(long)]
        all: bool,
    },
    /// Mechanics report for a materials DB entry (estimate-grade).
    MaterialCheck {
        /// Path to a materials TOML entry.
        path: PathBuf,
        /// Crease length for the stiffness estimate, meters.
        #[arg(long, default_value_t = 0.05)]
        crease_length_m: f64,
        /// Facet panel width for the buckling estimate, meters.
        #[arg(long, default_value_t = 0.05)]
        panel_width_m: f64,
        /// Plate-strip buckling coefficient (4.0 = clamped strip).
        #[arg(long, default_value_t = 4.0)]
        k_factor: f64,
        /// Run the dynamic radius gate at this r/t ratio.
        #[arg(long)]
        radius_rt: Option<f64>,
        /// Gate mode: exit non-zero unless the entry is measured.
        #[arg(long)]
        gate: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::FoldCheck { path, all } => match origami_cli::run_fold_check(&path, all) {
            Ok(reports) => {
                for report in &reports {
                    match &report.outcome {
                        origami_cli::CheckOutcome::Pass => {
                            println!("PASS {}", report.path.display());
                        }
                        origami_cli::CheckOutcome::Load(err) => {
                            println!("FAIL {} (load error: {err:?})", report.path.display());
                        }
                        origami_cli::CheckOutcome::Violations(violations) => {
                            println!("FAIL {}", report.path.display());
                            for v in violations {
                                println!("  {}", origami_cli::describe_violation(v));
                            }
                        }
                    }
                }
                if origami_cli::all_pass(&reports) {
                    println!("{} file(s), all green", reports.len());
                } else {
                    std::process::exit(1);
                }
            }
            Err(msg) => {
                eprintln!("{msg}");
                std::process::exit(1);
            }
        },
        Command::MaterialCheck {
            path,
            crease_length_m,
            panel_width_m,
            k_factor,
            radius_rt,
            gate,
        } => match origami_cli::run_material_check(
            &path,
            crease_length_m,
            panel_width_m,
            k_factor,
            radius_rt,
            gate,
        ) {
            Ok(report) => {
                for line in &report.lines {
                    println!("{line}");
                }
                if !report.gate_ok {
                    std::process::exit(1);
                }
            }
            Err(msg) => {
                eprintln!("{msg}");
                std::process::exit(1);
            }
        },
    }
}
