use crate::registry;
use dialoguer::Confirmation;
use itertools::Itertools;
use std::fs;
use std::path;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct ExecutionPlan {
    pub dirs_to_create: Vec<PathBuf>,
    pub moves: Vec<(PathBuf, PathBuf)>,
}
pub struct ExecutionOptions {
    pub force_overwrite: bool,
    pub no_prompts: bool,
}

pub struct Executor {
    registry: registry::Registry,
}

impl Executor {
    pub fn new(registry: registry::Registry) -> Executor {
        Executor { registry: registry }
    }
    pub fn plan(&self, input: String, fmt: String) -> ExecutionPlan {
        let mut mapped: Vec<(PathBuf, PathBuf)> = WalkDir::new(input)
            .into_iter()
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap())
            .filter(|ent| !ent.path().is_dir())
            .map(|ent| {
                let f = self.registry.format(&fmt, &ent);

                (ent.path().to_owned(), f)
            })
            .filter(|(_, res)| res.is_ok())
            .map(|(ent, res)| (ent, res.unwrap()))
            .map(|(ent, s)| (ent, path::Path::new(&s).to_owned()))
            .collect();
        mapped.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut missing_dirs: Vec<path::PathBuf> = mapped
            .iter()
            .filter(|(_, p)| p.parent().is_some())
            .map(|(_, p)| {
                return p.parent().unwrap();
            })
            .unique()
            .filter(|p| !p.exists())
            .map(|p| path::PathBuf::from(p))
            .collect();
        missing_dirs.sort_by(|a, b| a.cmp(b));

        return ExecutionPlan {
            dirs_to_create: missing_dirs,
            moves: mapped,
        };
    }
    pub fn validate(&self, plan: &ExecutionPlan) -> Result<bool, String> {
        let sources: Vec<&PathBuf> = plan.moves.iter().map(|(x, _)| x).unique().collect();
        let targets: Vec<&PathBuf> = plan.moves.iter().map(|(_, x)| x).unique().collect();
        if targets.iter().find(|t| t.to_str().unwrap() == "").is_some() {
            return Err("Found an empty target.".to_string());
        }

        if sources.len() == targets.len() {
            return Ok(true);
        }

        return Err(format!("Source and target file counts aren't equal: {} file(s) for source, {} file(s) for target.", sources.len(), targets.len()));
    }
    pub fn explain(&self, plan: &ExecutionPlan) -> String {
        return format!(
            "This will create {} dir(s) and move {} file(s)",
            plan.dirs_to_create.len(),
            plan.moves.len()
        );
    }
    pub fn execute(&self, plan: &ExecutionPlan, opts: ExecutionOptions) {
        let q = format!("{}. Continue?", self.explain(plan));

        if opts.no_prompts
            || Confirmation::new(&q)
                .default(false)
                .show_default(true)
                .interact()
                .unwrap()
        {
            plan.dirs_to_create.iter().for_each(|target_dir| {
                let _ = fs::create_dir_all(&target_dir);
            });
            plan.moves.iter().for_each(|(from, to)| {
                if to.exists() {
                    if opts.force_overwrite
                        || Confirmation::new(&format!("File {:?} exists. Overwrite?", to))
                            .default(false)
                            .show_default(true)
                            .interact()
                            .unwrap()
                    {
                        let _ = fs::rename(&from, &to);
                    } else {
                        // log
                        println!("Skipped {:?}.", to)
                    }
                } else {
                    let _ = fs::rename(&from, &to);
                }
            })
        } else {
            // log
            println!("Aborted without doing anything.")
        }
    }
}
