use slog::{error, info, o, Drain, Fuse, Logger};

use std::sync::Mutex;

use chrono::prelude::*;

use clap::{App, Arg};
use itertools::Itertools;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use rawsort::executor::{ExecutionOptions, Executor};
use rawsort::registry::Registry;

fn main() {
    let mut reg = Registry::new();
    reg.add(
        "[year]".to_string(),
        "Year (Photo date)".to_string(),
        |exif, _| exif.get_date().year().to_string(),
    );

    reg.add(
        "[month]".to_string(),
        "Month (Photo date)".to_string(),
        |exif, _| exif.get_date().month().to_string(),
    );
    reg.add(
        "[day]".to_string(),
        "Day (Photo date)".to_string(),
        |exif, _| exif.get_date().day().to_string(),
    );
    reg.add(
        "[hour]".to_string(),
        "Hour (Photo date)".to_string(),
        |exif, _| exif.get_date().hour().to_string(),
    );
    reg.add(
        "[minute]".to_string(),
        "Minute (Photo date)".to_string(),
        |exif, _| exif.get_date().minute().to_string(),
    );
    reg.add(
        "[second]".to_string(),
        "Second (Photo date)".to_string(),
        |exif, _| exif.get_date().second().to_string(),
    );
    reg.add(
        "[ext]".to_string(),
        "File extension only".to_string(),
        |_, ent| {
            return ent
                .path()
                .extension()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
        },
    );
    reg.add(
        "[filename]".to_string(),
        "Full file name, including ext".to_string(),
        |_, ent| {
            return ent
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
        },
    );

    let matches = App::new("rawsort")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Dotan N. <jondotan@gmail.com>")
        .about("Sort RAW and other standard photo formats.")
        .arg(
            Arg::with_name("out")
                .short("o")
                .long("out")
                .value_name("OUT")
                .default_value("photos/[year]/[month]/[day]/[filename]")
                .help(&format!(
                    "A directory layout and file format to sort photos by.\n\nFormats:\n{}",
                    reg.describe()
                        .iter()
                        .map(|(k, d)| format!("\t{}\t{}\n", k, d))
                        .join("")
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input folder to use.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .help("Force file overwrites."),
        )
        .arg(
            Arg::with_name("dryrun")
                .short("d")
                .long("dryrun")
                .help("Dry run (shows a report of whats going to happen)."),
        )
        .arg(
            Arg::with_name("json")
                .short("")
                .long("json")
                .help("Log using JSON (for integration with other tools)."),
        )
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .value_name("WATCH_DIR")
                .takes_value(true)
                .help("Watch input directory and run if files are added."),
        )
        .arg(
            Arg::with_name("yes")
                .short("y")
                .long("yes")
                .help("Answer automatice 'yes' to all prompts."),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity."),
        )
        .get_matches();

    let fmt = matches
        .value_of("out")
        .unwrap_or("[year]/[month]/[day]/[filename]");
    let input = matches.value_of("INPUT").unwrap_or(".");
    let force = matches.is_present("force");
    let yes = matches.is_present("yes");
    let dryrun = matches.is_present("dryrun");
    let watch_mode = matches.is_present("watch");
    let watch_dir = matches.value_of("watch").unwrap_or(input);
    let exec = Executor::new(reg);

    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let console = slog::Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!());

    let structured = slog::Logger::root(
        Mutex::new(slog_json::Json::default(std::io::stderr())).map(slog::Fuse),
        o!("version" => env!("CARGO_PKG_VERSION")),
    );

    let log = if matches.is_present("json") {
        structured
    } else {
        console
    };

    let run = || {
        let plan = exec.plan(input.to_string(), fmt.to_string());
        let res = exec.validate(&plan);
        match res {
            Err(s) => {
                error!(log, "Execution is not valid, aborting. Did you check your formatting rules?"; "error" => s);
                return;
            }
            Ok(_) => {}
        }
        if dryrun {
            plan.moves.iter().for_each(
                |(from, to)| info!(log, "move"; "from"=>from.to_str(), "to"=>to.to_str()),
            );
            plan.dirs_to_create
                .iter()
                .for_each(|d| info!(log, "create directory"; "directory"=> d.to_str()));
        } else {
            exec.execute(
                &plan,
                ExecutionOptions {
                    force_overwrite: force,
                    no_prompts: yes,
                },
            );
        }
    };
    run();

    if watch_mode {
        info!(log, "Watching"; "directory"=>watch_dir);
        if let Err(e) = watch(watch_dir, run) {
            error!(log, "watch error"; "error" => e.to_string())
        }
    }
}

fn watch<P: AsRef<Path>, F>(path: P, mut run: F) -> notify::Result<()>
where
    F: FnMut(),
{
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    // This is a simple loop, but you may want to use more complex logic here,
    // for example to handle I/O.
    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Create(event)) => {
                println!("Triggering because of: {:?}", event);

                run()
            }
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
