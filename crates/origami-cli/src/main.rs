//! FOLD validation CLI: structure + flat-foldability gates, materials
//! reports, and folded-state simulation. Composes on `origami-core`; the
//! logic lives in `lib.rs` so it stays testable.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Validates crease patterns, materials entries, and folding behavior.
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
    /// Fold a crease pattern with the bar-and-hinge solver.
    FoldSim {
        /// Path to a .fold file.
        path: PathBuf,
        /// Fold depth in [-1, 1] (targets = sign * percent * pi).
        #[arg(long, default_value_t = 0.5)]
        fold_percent: f64,
        /// Hard cap on integration steps.
        #[arg(long, default_value_t = 3000)]
        max_iterations: u32,
        /// Per-projection relaxation factor (0 to 1].
        #[arg(long, default_value_t = 0.9)]
        relax: f64,
        /// Per-vertex displacement cap per sweep, model units.
        #[arg(long, default_value_t = 0.05)]
        step_cap: f64,
        /// Emit the final vertices and report as JSON.
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let outcome = match cli.command {
        Command::FoldCheck { path, all } => {
            origami_cli::run_fold_check(&path, all).map(|reports| {
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
                println!("{} file(s), all green", reports.len());
                origami_cli::all_pass(&reports)
            })
        }
        Command::MaterialCheck {
            path,
            crease_length_m,
            panel_width_m,
            k_factor,
            radius_rt,
            gate,
        } => origami_cli::run_material_check(
            &path,
            crease_length_m,
            panel_width_m,
            k_factor,
            radius_rt,
            gate,
        )
        .map(|report| {
            for line in &report.lines {
                println!("{line}");
            }
            report.gate_ok
        }),
        Command::FoldSim {
            path,
            fold_percent,
            max_iterations,
            relax,
            step_cap,
            json,
        } => origami_cli::run_fold_sim(&path, fold_percent, max_iterations, relax, step_cap, json)
            .map(|out| {
                for line in &out.lines {
                    println!("{line}");
                }
                out.converged
            }),
    };
    match outcome {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(1);
        }
    }
}
