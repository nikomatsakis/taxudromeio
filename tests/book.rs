use std::{
    fs,
    path::{Path, PathBuf},
};

use anchor_book::{
    Chapter,
    checking::{self, Outcome},
    read_records,
    roadmaps::read_roadmaps,
};

fn markdown(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            markdown(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "md")
            && path.file_name().unwrap() != "SUMMARY.md"
        {
            paths.push(path);
        }
    }
}

#[test]
fn book_structure_records_and_roadmaps_conform_to_the_pinned_engine() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let report =
        checking::check(&root.join("anchorbook.toml")).unwrap_or_else(|error| error.into_report());
    assert_eq!(
        report.outcome(),
        Outcome::Passed,
        "{}",
        anchor_book::adapters::check::format_report(&report)
    );
    let mut paths = Vec::new();
    markdown(&root.join("book"), &mut paths);
    paths.sort();
    let contents = paths
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>();
    let chapters = paths
        .iter()
        .zip(&contents)
        .map(|(path, text)| Chapter::new(path, text))
        .collect::<Vec<_>>();
    let records = read_records(&chapters).unwrap();
    assert!(!records.is_empty());
    let roadmaps = chapters
        .into_iter()
        .filter(|c| {
            c.path
                .starts_with(root.join("book/implementation-roadmaps"))
        })
        .collect::<Vec<_>>();
    assert!(!read_roadmaps(&roadmaps, &records).unwrap().is_empty());
}

/// Exercises the installed engine, including the gate omitted by the former adapter.
#[test]
#[ignore = "requires scripts/setup-book-tools.sh"]
fn installed_book_tools_match_the_pin_and_gate_rendering() {
    use std::process::{Command, Output};

    fn copy_tree(source: &Path, target: &Path) {
        fs::create_dir_all(target).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let destination = target.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &destination);
            } else {
                fs::copy(entry.path(), destination).unwrap();
            }
        }
    }
    fn successful(output: Output) -> String {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bin = root.join("target/book-tools/bin");
    let engine = bin.join("anchorbook");
    let lock: toml::Value = toml::from_str(include_str!("../Cargo.lock")).unwrap();
    let package = lock["package"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["name"].as_str() == Some("anchor-book"))
        .unwrap();
    let revision = package["source"]
        .as_str()
        .unwrap()
        .rsplit_once('#')
        .unwrap()
        .1;
    let version = successful(Command::new(&engine).arg("--version").output().unwrap());
    assert!(
        version.trim_end().ends_with(&format!("({revision})")),
        "{version}"
    );
    let configuration = anchor_book::configuration::load(&root.join("anchorbook.toml")).unwrap();
    let contract = successful(
        Command::new(&engine)
            .args(["contract", "--config"])
            .arg(root.join("anchorbook.toml"))
            .output()
            .unwrap(),
    );
    assert_eq!(
        contract,
        anchor_book::contract::for_phase(configuration.phase())
    );

    let fixture = tempfile::tempdir().unwrap();
    copy_tree(&root.join("book"), &fixture.path().join("book"));
    for name in ["anchorbook.toml", "mermaid.min.js", "mermaid-init.js"] {
        fs::copy(root.join(name), fixture.path().join(name)).unwrap();
    }
    let mut config: toml::Value =
        toml::from_str(&fs::read_to_string(root.join("book.toml")).unwrap()).unwrap();
    config["preprocessor"]["anchorbook"]["command"] = toml::Value::String(format!(
        "{} md",
        shlex::try_quote(engine.to_str().unwrap()).unwrap()
    ));
    config["preprocessor"]["mermaid"]["command"] = toml::Value::String(
        shlex::try_quote(bin.join("mdbook-mermaid").to_str().unwrap())
            .unwrap()
            .into_owned(),
    );
    fs::write(
        fixture.path().join("book.toml"),
        toml::to_string(&config).unwrap(),
    )
    .unwrap();
    let build = || {
        Command::new(bin.join("mdbook"))
            .arg("build")
            .current_dir(fixture.path())
            .output()
            .unwrap()
    };
    successful(build());
    let output = fixture.path().join("target/anchorbook");
    let roadmap = "implementation-roadmaps/initial-server";
    let source = fs::read_to_string(fixture.path().join(format!("book/{roadmap}.md"))).unwrap();
    let rendered = fs::read_to_string(output.join(format!("{roadmap}.html"))).unwrap();
    assert_eq!(
        rendered.matches("<section class=\"ab-milestone\">").count(),
        source.matches("\n## Milestone ").count()
    );
    assert!(rendered.contains("ab-milestone-evidence"));
    let diagram = fs::read_to_string(output.join("design/index.html")).unwrap();
    assert!(diagram.contains("class=\"mermaid\""));
    assert!(!diagram.contains("language-mermaid"));

    let config_path = fixture.path().join("anchorbook.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    fs::write(
        &config_path,
        original.replace("design/index.md", "design/not-present.md"),
    )
    .unwrap();
    fs::remove_dir_all(&output).unwrap();
    let failure = build();
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("structure.role-missing"));
    assert!(!output.join("design/index.html").exists());

    fs::write(&config_path, original).unwrap();
    fs::write(
        fixture.path().join(format!("book/{roadmap}.md")),
        source.replacen("`operation-filter`", "`nonexistent-criterion`", 1),
    )
    .unwrap();
    let failure = build();
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("nonexistent-criterion"));
}
