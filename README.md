# Rawsort

A simple but powerful RAW photo import tool built to have amazing performance and integrate with photography workflows.

Having this as the import tool in my workflow meant I no longer needed to pay for Lightroom subscription.


## Quick Start

Rawsort is primarily a command line tool (but it was built to integrate with GUI tools). Here are some workflow ideas.


(All examples are with the `-d` (dryrun) flag, feel free to remove when ready.)

### One time import

Sort all files from given source (`/Volumes/SD_CARD`)

```
$ rawsort -d --out 'out/[year]/[day]/[hour]/[filename]' /Volumes/SD_CARD
```

### Watch mode

Watch for when a new media is entered (`-w /Volumes`) and automatically run sorting.

```
$ rawsort -d --out 'out/[year]/[day]/[hour]/[filename]' -w /Volumes /Volumes/SD_CARD
```

### Use JSON (for integration)

To integrate with other tools, run the same command with `-j`, makes tracking what the tool does very easy.

```
$ rawsort -j -d --out 'out/[year]/[day]/[hour]/[filename]' /Volumes/SD_CARD
```

### Other options

Take a look at `--help`:

```
Dotan N. <jondotan@gmail.com>
Sort RAW and other standard photo formats.

USAGE:
    rawsort [FLAGS] [OPTIONS] <INPUT>

FLAGS:
    -d, --dryrun     Dry run (shows a report of whats going to happen).
    -f, --force      Force file overwrites.
    -h, --help       Prints help information
        --json       Log using JSON (for integration with other tools).
    -v               Sets the level of verbosity.
    -V, --version    Prints version information
    -y, --yes        Answer automatic 'yes' to all prompts.

OPTIONS:
    -o, --out <OUT>            A directory layout and file format to sort photos by.

                               Formats:
                               	[year]	Year (Photo date)
                               	[filename]	Full file name, including ext
                               	[minute]	Minute (Photo date)
                               	[second]	Second (Photo date)
                               	[month]	Month (Photo date)
                               	[day]	Day (Photo date)
                               	[hour]	Hour (Photo date)
                               	[ext]	File extension only
                                [default: photos/[year]/[month]/[day]/[filename]]
    -w, --watch <WATCH_DIR>    Watch input directory and run if files are added.

ARGS:
    <INPUT>    Sets the input folder to use.

```

## Thanks:

To all [Contributors](https://github.com/jondot/rawsort/graphs/contributors) - you make this happen, thanks!

# Copyright

Copyright (c) 2018 [@jondot](http://twitter.com/jondot). See [LICENSE](LICENSE.txt) for further details.