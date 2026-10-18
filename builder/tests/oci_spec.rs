mod support;

#[path = "oci_spec/compression.rs"]
mod compression;
#[path = "oci_spec/config.rs"]
mod config;
#[path = "oci_spec/descriptor.rs"]
mod descriptor;
#[path = "oci_spec/index.rs"]
mod index;
#[path = "oci_spec/layout.rs"]
mod layout;
#[path = "oci_spec/manifest.rs"]
mod manifest;

use support::{Layout, amd64, build};

fn sample() -> Layout {
    build(&[amd64("v1"), amd64("v3")])
}

fn table_columns(line: &str) -> Vec<&str> {
    line.split('|')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .collect()
}

fn documented_platforms() -> Vec<(String, String)> {
    let md = include_str!("../../docs/user/targets.md");
    let mut rows = Vec::new();
    for line in md.lines() {
        let columns = table_columns(line);
        if columns.len() != 2 {
            continue;
        }
        if columns[0] == "architecture" || columns[0].starts_with('-') {
            continue;
        }
        rows.push((columns[0].to_string(), columns[1].to_string()));
    }
    rows
}

fn platform_statuses() -> Vec<(String, String, String)> {
    let md = include_str!("../../docs/dev/oci.md");
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in md.lines() {
        let columns = table_columns(line);
        if columns.len() == 3 && columns[0] == "architecture" && columns[2] == "status" {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if columns.len() != 3 {
            break;
        }
        if columns[0].starts_with('-') {
            continue;
        }
        rows.push((
            columns[0].to_string(),
            columns[1].to_string(),
            columns[2].to_string(),
        ));
    }
    rows
}
