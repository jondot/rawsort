extern crate clap;
extern crate dialoguer;
extern crate itertools;
extern crate notify;
extern crate rawsort;
extern crate walkdir;
use chrono::prelude::*;
use itertools::Itertools;
extern crate chrono;
use clap::{App, Arg};
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

    let matches = App::new("Sorter")
        .version("1.0")
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

    let run = || {
        let plan = exec.plan(input.to_string(), fmt.to_string());
        let res = exec.validate(&plan);
        match res {
            Err(s) => {
                println!("Execution is not valid, aborting. Did you check your formatting rules?");
                println!("Error: {}", s);
                return
            }
            Ok(_)=>{}
        }
        if dryrun {
            plan.moves
                .iter()
                .for_each(|(from, to)| println!("{:?} -> {:?}", from, to));
            println!("Directories to be created:");
            plan.dirs_to_create.iter().for_each(|d| println!("{:?}", d));
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
        println!("Now watching '{}'...", watch_dir);
        if let Err(e) = watch(watch_dir, run) {
            println!("watch error: {:?}", e)
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
    let mut watcher: RecommendedWatcher = try!(Watcher::new(tx, Duration::from_secs(2)));

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    try!(watcher.watch(path, RecursiveMode::NonRecursive));

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
