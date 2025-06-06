use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

fn main() {
    let dir = tempfile::tempdir().unwrap();
    let layout_file = dir.path().join("layout.kdl");
    let vcs_type = VcsType::get();
    let tree_pane = to_kdl_pane(tree(vcs_type));
    let status_pane = to_kdl_pane(status(vcs_type));
    fs::write(
        &layout_file,
        format!(
            r#"
                layout {{
                    pane split_direction="vertical" {{
                        pane {{
                            command "zsh"
                        }}

                        pane split_direction="horizontal" {{
                            {tree_pane}
                            {status_pane}
                        }}
                    }}
                }}
            "#
        ),
    )
    .unwrap();
    Command::new("zellij")
        .arg("--layout")
        .arg(&layout_file)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
}

fn to_kdl_pane(input: (String, Vec<String>)) -> String {
    let (command, args) = input;
    if args.is_empty() {
        format!(r#"pane {{ command "{command}"; }}"#)
    } else {
        let args = args
            .into_iter()
            .map(|arg| format!("\"{arg}\""))
            .collect::<Vec<_>>()
            .join(" ");
        format!(r#"pane {{ command "{command}"; args {args}; }}"#)
    }
}

#[derive(Debug, Clone, Copy)]
enum VcsType {
    None,
    Git,
    Jujutsu,
}

impl VcsType {
    fn get() -> VcsType {
        if Path::new("./.jj").exists() {
            VcsType::Jujutsu
        } else if Path::new("./.git").exists() {
            VcsType::Git
        } else {
            VcsType::None
        }
    }
}

fn tree(vcs_type: VcsType) -> (String, Vec<String>) {
    match vcs_type {
        VcsType::None => ("echo".to_string(), vec!["no vcs detected".to_string()]),
        VcsType::Git => ("git-watch-tree".to_owned(), Vec::new()),
        VcsType::Jujutsu => (
            "/usr/bin/watch".to_string(),
            vec!["jj", "log", "--color", "always"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
        ),
    }
}

fn status(vcs_type: VcsType) -> (String, Vec<String>) {
    match vcs_type {
        VcsType::None => ("echo".to_string(), vec!["no vcs detected".to_string()]),
        VcsType::Git => (
            "/usr/bin/watch".to_string(),
            "--color git -c color.ui=always status"
                .split_whitespace()
                .map(ToString::to_string)
                .collect(),
        ),
        VcsType::Jujutsu => (
            "/usr/bin/watch".to_string(),
            vec![
                "bash",
                "-c",
                "\\\"jj status --color always && jj diff --color always\\\"",
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
        ),
    }
}
