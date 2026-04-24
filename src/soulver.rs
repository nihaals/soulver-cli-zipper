use std::{ffi::OsStr, path::Path, process::Command};

use anyhow::{Result, bail, ensure};

fn run_raw_soulver(file: &str, soulver_path: Option<&Path>) -> Result<String> {
    let soulver_command: &OsStr = match soulver_path {
        Some(path) => {
            if !path.is_file() {
                bail!("soulver_path is not a file");
            }
            path.as_os_str()
        }
        None => OsStr::new("soulver"),
    };
    let output = Command::new(soulver_command).arg(file).output()?;
    if !output.status.success() {
        bail!("soulver exited with non-zero exit code");
    }
    let stdout = str::from_utf8(&output.stdout)?;
    let stdout_no_trailing = stdout.strip_suffix('\n').unwrap_or(stdout);
    Ok(stdout_no_trailing.to_owned())
}

fn get_number_of_initial_newlines<I, S>(lines: I) -> usize
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    lines
        .into_iter()
        .take_while(|line| {
            let line_str = line.as_ref();
            line_str.is_empty() || line_str.starts_with('#') || line_str.starts_with("//")
        })
        .count()
}

pub fn run_soulver(file: &str, soulver_path: Option<&Path>) -> Result<String> {
    let trimmed_input = file.trim_end();
    let mut output = run_raw_soulver(trimmed_input, soulver_path)?;

    let initial_newlines = get_number_of_initial_newlines(trimmed_input.lines());
    if initial_newlines > 0 {
        output.insert_str(0, &"\n".repeat(initial_newlines));
    }

    Ok(output)
}

pub fn run_soulver_zipped(
    file: &str,
    aligned_bars: bool,
    soulver_path: Option<&Path>,
) -> Result<String> {
    let trimmed_input = file.trim_end();
    let output = run_soulver(trimmed_input, soulver_path)?;
    let output_lines: Vec<String> = output.lines().map(|line| line.to_owned()).collect();
    let input_lines: Vec<&str> = trimmed_input.lines().collect();
    let longest_input_line_length = if aligned_bars {
        input_lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
    } else {
        0
    };

    let mut out = String::with_capacity(trimmed_input.len() + output.len());
    ensure!(input_lines.len() == output_lines.len());
    for (input_line, output_line) in input_lines.iter().zip(output_lines.iter()) {
        if output_line.is_empty() {
            if !aligned_bars && input_line.is_empty() {
                out.push_str("|\n");
            } else {
                out.push_str(&format!(
                    "{input_line:<width$} |\n",
                    width = longest_input_line_length,
                ));
            }
        } else {
            out.push_str(&format!(
                "{input_line:<width$} | {output_line}\n",
                width = longest_input_line_length,
            ));
        }
    }
    if out.ends_with('\n') {
        out.pop();
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    fn soulver_path() -> Option<PathBuf> {
        std::env::var("SOULVER_PATH").ok().map(PathBuf::from)
    }

    fn run_raw_soulver(file: &str) -> Result<String> {
        super::run_raw_soulver(file, soulver_path().as_deref())
    }

    fn run_soulver(file: &str) -> Result<String> {
        super::run_soulver(file, soulver_path().as_deref())
    }

    fn run_soulver_zipped(file: &str, aligned_bars: bool) -> Result<String> {
        super::run_soulver_zipped(file, aligned_bars, soulver_path().as_deref())
    }

    #[test]
    fn test_run_raw_soulver_variable() {
        assert_eq!(run_raw_soulver("Foo = 1\nFoo + 2").unwrap(), "1\n3")
    }

    #[test]
    fn test_run_raw_soulver_newlines() {
        assert_eq!(run_raw_soulver("\n1\n\n2").unwrap(), "1\n\n2")
    }

    #[test]
    fn test_run_raw_soulver_headings() {
        assert_eq!(run_raw_soulver("# Foo\n1\n# Bar\n2").unwrap(), "1\n\n2")
    }

    #[test]
    fn test_run_raw_soulver_trailing_newlines() {
        assert_eq!(run_raw_soulver("1\n\n\n").unwrap(), "1\n\n")
    }

    #[test]
    fn test_run_raw_soulver_trailing_headers() {
        assert_eq!(run_raw_soulver("1\n# Foo\n# Bar").unwrap(), "1\n\n")
    }

    #[test]
    fn test_run_raw_soulver_no_end_only_newlines_1() {
        assert_eq!(run_raw_soulver("\n").unwrap(), "")
    }

    #[test]
    fn test_run_raw_soulver_no_end_only_newlines_2() {
        assert_eq!(run_raw_soulver("\n\n").unwrap(), "")
    }

    #[test]
    fn test_run_raw_soulver_no_end_only_newlines_3() {
        assert_eq!(run_raw_soulver("\n\n\n").unwrap(), "")
    }

    /// Calculate the number of missing leading newlines using `soulver`.
    fn get_correct_number_of_initial_newlines(lines: &[&str]) -> usize {
        for line_number_upper_bound in 1..=lines.len() {
            let lines_to_check = &lines[0..line_number_upper_bound];
            let test_input = format!("1\n{}\n1", lines_to_check.join("\n"));
            let result = run_raw_soulver(&test_input).unwrap();
            // Expected assuming all lines generate no output
            let expected = format!("1{}\n1", "\n".repeat(lines_to_check.len()));
            if result == expected {
                continue;
            }
            return line_number_upper_bound - 1;
        }

        lines.len()
    }

    #[test]
    fn test_get_number_of_initial_newlines_mix_end() {
        let expected = 3;
        let lines = &["", "# Foo", "// Bar", "1"];
        assert_eq!(get_correct_number_of_initial_newlines(lines), expected);
        assert_eq!(get_number_of_initial_newlines(lines), expected);
    }

    #[test]
    fn test_get_number_of_initial_newlines_mix_no_end() {
        let expected = 3;
        let lines = &["", "# Foo", "// Bar"];
        assert_eq!(get_correct_number_of_initial_newlines(lines), expected);
        assert_eq!(get_number_of_initial_newlines(lines), expected);
    }

    #[test]
    fn test_get_number_of_initial_newlines_only_newlines_no_end() {
        let expected = 3;
        let lines = &["", "", ""];
        assert_eq!(get_correct_number_of_initial_newlines(lines), expected);
        assert_eq!(get_number_of_initial_newlines(lines), expected);
    }

    #[test]
    fn test_run_soulver_variable() {
        assert_eq!(run_soulver("Foo = 1\nFoo + 2").unwrap(), "1\n3")
    }

    #[test]
    fn test_run_soulver_newlines() {
        assert_eq!(run_soulver("\n1\n\n2").unwrap(), "\n1\n\n2")
    }

    #[test]
    fn test_run_soulver_headings() {
        assert_eq!(run_soulver("# Foo\n1\n# Bar\n2").unwrap(), "\n1\n\n2")
    }

    #[test]
    fn test_run_soulver_no_end() {
        assert_eq!(run_soulver("\n# Foo\n// Bar\n").unwrap(), "\n\n\n")
    }

    #[test]
    fn test_run_soulver_no_end_only_newlines_1() {
        assert_eq!(run_soulver("\n").unwrap(), "")
    }

    #[test]
    fn test_run_soulver_no_end_only_newlines_2() {
        assert_eq!(run_soulver("\n\n").unwrap(), "")
    }

    #[test]
    fn test_run_soulver_no_end_only_newlines_3() {
        assert_eq!(run_soulver("\n\n\n").unwrap(), "")
    }

    #[test]
    fn test_run_soulver_trailing_newlines_2() {
        assert_eq!(run_soulver("1\n\n").unwrap(), "1")
    }

    #[test]
    fn test_run_soulver_trailing_newlines_3() {
        assert_eq!(run_soulver("1\n\n\n").unwrap(), "1")
    }

    #[test]
    fn test_run_soulver_zipped_variable_aligned() {
        assert_eq!(
            run_soulver_zipped("Foo = 1\nFoo + 2", true).unwrap(),
            "Foo = 1 | 1\nFoo + 2 | 3",
        )
    }

    #[test]
    fn test_run_soulver_zipped_newlines_aligned() {
        assert_eq!(
            run_soulver_zipped("\n1\n\n2", true).unwrap(),
            "  |\n1 | 1\n  |\n2 | 2"
        )
    }

    #[test]
    fn test_run_soulver_zipped_newlines_unaligned() {
        assert_eq!(
            run_soulver_zipped("\n1\n\n2", false).unwrap(),
            "|\n1 | 1\n|\n2 | 2"
        )
    }

    #[test]
    fn test_run_soulver_zipped_headings_aligned() {
        assert_eq!(
            run_soulver_zipped("# Foo\n1\n# Bar\n2", true).unwrap(),
            "# Foo |\n1     | 1\n# Bar |\n2     | 2",
        )
    }

    #[test]
    fn test_run_soulver_zipped_headings_unaligned() {
        assert_eq!(
            run_soulver_zipped("# Foo\n1\n# Bar\n2", false).unwrap(),
            "# Foo |\n1 | 1\n# Bar |\n2 | 2",
        )
    }

    #[test]
    fn test_run_soulver_zipped_no_end_aligned() {
        assert_eq!(
            run_soulver_zipped("\n# Foo\n// Bar\n", true).unwrap(),
            "       |\n# Foo  |\n// Bar |",
        )
    }

    #[test]
    fn test_run_soulver_zipped_no_end_unaligned() {
        assert_eq!(
            run_soulver_zipped("\n# Foo\n// Bar\n", false).unwrap(),
            "|\n# Foo |\n// Bar |",
        )
    }

    #[test]
    fn test_run_soulver_zipped_pound_sign_aligned() {
        assert_eq!(
            run_soulver_zipped("# Foo\nBar = £1", true).unwrap(),
            "# Foo    |\nBar = £1 | £1.00",
        )
    }

    #[test]
    fn test_run_soulver_zipped_pound_sign_unaligned() {
        assert_eq!(
            run_soulver_zipped("# Foo\nBar = £1", false).unwrap(),
            "# Foo |\nBar = £1 | £1.00",
        )
    }

    #[test]
    fn test_run_soulver_zipped_trailing_newlines_1_aligned() {
        assert_eq!(run_soulver_zipped("1\n", true).unwrap(), "1 | 1")
    }

    #[test]
    fn test_run_soulver_zipped_trailing_newlines_1_unaligned() {
        assert_eq!(run_soulver_zipped("1\n", false).unwrap(), "1 | 1")
    }

    #[test]
    fn test_run_soulver_zipped_trailing_newlines_2_aligned() {
        assert_eq!(run_soulver_zipped("1\n\n", true).unwrap(), "1 | 1")
    }

    #[test]
    fn test_run_soulver_zipped_trailing_newlines_2_unaligned() {
        assert_eq!(run_soulver_zipped("1\n\n", false).unwrap(), "1 | 1")
    }

    #[test]
    fn test_run_soulver_zipped_trailing_newlines_3_aligned() {
        assert_eq!(run_soulver_zipped("1\n\n\n", true).unwrap(), "1 | 1")
    }

    #[test]
    fn test_run_soulver_zipped_trailing_newlines_3_unaligned() {
        assert_eq!(run_soulver_zipped("1\n\n\n", false).unwrap(), "1 | 1")
    }
}
