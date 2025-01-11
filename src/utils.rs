use std::collections::BTreeMap;

use itertools::Itertools;

// start and end are inclusive
pub fn get_group_annotations(annotations: &BTreeMap<usize, String>, start: usize, end: usize) -> Option<String> {
    let mut grouped_comments = vec![];
    for row in start..end+1 {
        if let Some(annotation) = annotations.get(&row) {
            if let Some ((_, last_end, last_comment)) = grouped_comments.last_mut() {
                if row == *last_end + 1 && *last_comment == annotation {
                    *last_end = row;
                } else {
                    grouped_comments.push((row, row, annotation));
                }
            } else {
                grouped_comments.push((row, row, annotation));
            }
        }
    }
    if grouped_comments.is_empty() {
        None
    } else {
        Some(grouped_comments
            .iter()
            .map(|(start, end, comment)| {
                if *start == *end {
                    format!("  -- {start}: {comment}")
                } else {
                    format!("  -- {start}-{end}: {comment}")
                }
            })
            .join("\n"))
    }
}

pub fn group_values(column: &BTreeMap<usize, String>) -> Vec<(String, usize, Option<usize>)> {
    let mut res: Vec<(String, usize, Option<usize>)> = vec![];

    for (row, value) in column.iter() {
        if let Some((val, start, end)) = res.last_mut() {
            if val == value && end.unwrap_or(*start) + 1 == *row {
                *end = Some(*row);
            } else {
                res.push((value.clone(), *row, None));
            }
        } else {
            res.push((value.clone(), *row, None));
        }
    }

    res
}

pub fn make_lean_comment(text: &str) -> String {
    text
        .split("\n")
        .map(|line| format!("  --{line}"))
        .join("\n")
}

pub fn print_grouped_props(prefix: &str, final_name: &str, props: &[String], group_size: usize) {
    assert!(group_size > 1);
    let mut groups = vec![vec![]];

    for (idx, prop) in props.iter().enumerate() {
        let name = format!("{prefix}{idx}");
        println!("def {name} (c: ValidCircuit P P_Prime) : Prop :=");
        println!("  {prop}");
        groups[0].push((idx, idx, name));
        let mut i = 0;
        while i < groups.len() {
            if groups[i].len() >= group_size {
                let start = groups[i][0].0;
                let end = groups[i][groups[i].len()-1].1;
                let name = format!("{prefix}{start}_to_{end}");
                let body = groups[i]
                    .iter()
                    .map(|(_, _, name)| format!("{name} c"))
                    .join(" ∧ ");
                println!("def {name} (c: ValidCircuit P P_Prime) : Prop :=");
                println!("  {body}");
                if groups.len() == i+1 {
                    groups.push(vec![]);
                }
                groups[i+1].push((start, end, name));
                groups[i] = vec![];
            }
            i += 1;
        }
    }

    let final_body = groups
        .iter()
        .rev()
        .flatten()
        .map(|(_, _, name)| format!("{name} c"))
        .join(" ∧ ");

    let final_body = if final_body == "" {
        "true"
    } else {
        &final_body
    };

    println!("def {final_name} (c: ValidCircuit P P_Prime): Prop :=");
    println!("  {final_body}");
}

pub fn update_column_annotation(annotations: &mut BTreeMap<usize, (Option<String>, BTreeMap<usize, String>)>, col: usize, annotation: String) {
    let current = annotations.get_mut(&col);
    if let Some((column, _)) = current {
        *column = Some(annotation);
    } else {
        annotations.insert(col, (Some(annotation), BTreeMap::new()));
    }
}

pub fn update_row_annotation(annotations: &mut BTreeMap<usize, (Option<String>, BTreeMap<usize, String>)>, col: usize, row: usize, annotation: String) {
    let current = annotations.get_mut(&col);
    if let Some((_, rows)) = current {
        rows.insert(row, annotation);
    } else {
        let mut rows = BTreeMap::new();
        rows.insert(row, annotation);
        annotations.insert(col, (None, rows));
    }
}