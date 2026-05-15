use std::env;
use std::path::{Path, PathBuf};

use graphene_gen_bindings_rs::{
    default_options, generate_all, generate_chain, ChainBindingJob, GenerateOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = find_workspace_root(env::current_dir()?)?;
    let mut options = default_options(&workspace_root);

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--chain" => {
                let chain = args
                    .next()
                    .ok_or("--chain requires acta, swaplock, or all")?;
                if chain != "all" {
                    options.jobs.retain(|job| job.chain == chain);
                    if options.jobs.is_empty() {
                        return Err(format!("unknown chain: {chain}").into());
                    }
                }
            }
            "--manifest" => {
                let manifest = PathBuf::from(args.next().ok_or("--manifest requires a path")?);
                let crate_dir = PathBuf::from(args.next().ok_or(
                    "--manifest requires a following crate directory path for custom jobs",
                )?);
                let chain = manifest
                    .parent()
                    .and_then(|path| path.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or("custom")
                    .to_owned();
                options = GenerateOptions {
                    workspace_root: workspace_root.clone(),
                    jobs: vec![ChainBindingJob {
                        chain,
                        manifest: absolutize(&workspace_root, &manifest),
                        crate_dir: absolutize(&workspace_root, &crate_dir),
                    }],
                };
            }
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }

    if options.jobs.len() == 1 {
        generate_chain(&options.jobs[0])?;
    } else {
        generate_all(&options)?;
    }
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: graphene-gen-bindings-rs [--chain acta|swaplock|all]\n       graphene-gen-bindings-rs --manifest path/to/open-graphene.json path/to/output-crate"
    );
}

fn find_workspace_root(mut dir: PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("graphene/specs").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err("failed to find workspace root".into());
        }
    }
}

fn absolutize(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}
