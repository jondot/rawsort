extern crate rawsort;
use chrono::prelude::*;
extern crate chrono;
use rawsort::executor::{ExecutionPlan, Executor};
use rawsort::registry::Registry;
use std::path::{Path, PathBuf};

fn build_registry() -> Registry {
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

    reg
}
fn fixture(f: &'static str) -> String {
    Path::new(file!())
        .parent()
        .unwrap()
        .join("fixtures")
        .join(f)
        .to_str()
        .unwrap()
        .to_string()
}
fn moves(strs: &[(&'static str, &'static str)]) -> Vec<(PathBuf, PathBuf)> {
    strs.to_vec()
        .iter()
        .map(|(s1, s2)| (PathBuf::from(s1), PathBuf::from(s2)))
        .collect()
}
fn dirs(strs: &[&'static str]) -> Vec<PathBuf> {
    strs.to_vec().iter().map(|s| PathBuf::from(s)).collect()
}
fn make_plan(fixture_kind: &'static str, format: &'static str) -> (ExecutionPlan, Executor) {
    let reg = build_registry();
    let exec = Executor::new(reg);
    let plan = exec.plan(fixture(fixture_kind), format.to_string());
    return (plan, exec);
}
#[test]
fn it_detects_bad_formats() {
    let (plan, exec) = make_plan("simple", "");

    assert_eq!(
        plan.moves,
        moves(&[("tests/fixtures/simple/20171104-DSC_1233.NEF", "")])
    );
    assert_eq!(exec.validate(&plan).unwrap_err(), "Found an empty target.");
}

#[test]
fn it_will_not_format_almost_a_good_format() {
    let (plan, exec) = make_plan("simple", "year");

    assert_eq!(
        plan.moves,
        moves(&[("tests/fixtures/simple/20171104-DSC_1233.NEF", "year")])
    );
    assert_eq!(exec.validate(&plan).unwrap(), true);
}

#[test]
fn it_will_format_a_good_format() {
    let (plan, exec) = make_plan("simple", "[year]");

    assert_eq!(
        plan.moves,
        moves(&[("tests/fixtures/simple/20171104-DSC_1233.NEF", "2017")])
    );
}

#[test]
fn it_detects_potential_overwrites() {
    let (plan, exec) = make_plan("multiple", "[year]");
    assert_eq!(
        exec.validate(&plan).unwrap_err(),
        "Source and target file counts aren't equal: 2 file(s) for source, 1 file(s) for target."
    );
}

#[test]
fn it_plans_properly() {
    let (plan, exec) = make_plan(
        "multiple",
        "[year]/[month]/[day]/[hour]/[minute]/[second]/[filename].[ext]",
    );
    assert_eq!(
        plan.moves,
        moves(&[
            (
                "tests/fixtures/multiple/20171104-DSC_1236.JPG",
                "2017/11/4/12/45/23/20171104-DSC_1236.JPG.JPG"
            ),
            (
                "tests/fixtures/multiple/20171104-DSC_1237.JPG",
                "2017/11/4/12/45/34/20171104-DSC_1237.JPG.JPG"
            )
        ])
    );
    assert_eq!(
        plan.dirs_to_create,
        dirs(&["2017/11/4/12/45/23", "2017/11/4/12/45/34"])
    );
}

#[cfg(nightly)]
fn bench_make_plan(b: &mut Bencher) {
    #![feature(test)]
    #[bench]
    use test::Bencher;
    extern crate test;
    b.iter(|| {
        make_plan(
            "multiple",
            "[year]/[month]/[day]/[hour]/[minute]/[second]/[filename].[ext]",
        )
    });
}
