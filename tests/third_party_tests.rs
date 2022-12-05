use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use php_parser_rs::prelude::Lexer;
use php_parser_rs::prelude::Parser;

#[test]
fn third_party_1_php_standard_library() {
    test_repository(
        "php-standard-library",
        "https://github.com/azjezz/psl.git",
        "2.2.x",
        &["src", "tests", "examples"],
        &[],
    );
}

#[test]
fn third_party_2_laravel_framework() {
    test_repository(
        "laravel-framework",
        "https://github.com/laravel/framework",
        "9.x",
        &["src", "tests"],
        &[],
    );
}

#[test]
fn third_party_3_symfony_framework() {
    test_repository(
        "symfony-framework",
        "https://github.com/symfony/symfony",
        "6.3",
        &["src/Symfony"],
        &[
            "src/Symfony/Bridge/ProxyManager/Tests/LazyProxy/PhpDumper/Fixtures/proxy-implem.php",
            "src/Symfony/Component/Config/Tests/Fixtures/ParseError.php",
            // FIXME: Remove this one once I've found the energy to sort out heredocs / nowdocs.
            "src/Symfony/Component/DependencyInjection/LazyProxy/PhpDumper/LazyServiceDumper.php",
            "src/Symfony/Component/Cache/Tests/Traits/RedisProxiesTest.php",
            // FIXME: Remove this once we can support (A&B)|C DNF types.
            "src/Symfony/Component/DependencyInjection/Tests/Fixtures/includes/compositetype_classes.php",
            // FIXME: Remove these once we can support arbitrary opening and closing tags.
            "src/Symfony/Component/ErrorHandler/Resources/views/exception.html.php",
            "src/Symfony/Component/ErrorHandler/Resources/views/exception_full.html.php",
            "src/Symfony/Component/ErrorHandler/Resources/views/logs.html.php",
            "src/Symfony/Component/ErrorHandler/Resources/views/trace.html.php",
            "src/Symfony/Component/ErrorHandler/Resources/views/traces.html.php",
            "src/Symfony/Component/ErrorHandler/Resources/views/traces_text.html.php"
        ],
    );
}

fn test_repository(
    name: &str,
    repository: &str,
    branch: &str,
    directories: &[&str],
    ignore: &[&str],
) {
    let out_dir = env::var_os("OUT_DIR").expect("failed to get OUT_DIR");
    let out_path = PathBuf::from(out_dir).join(name);

    if !out_path.exists() {
        let output = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("-b")
            .arg(branch)
            .arg(repository)
            .arg(&out_path)
            .output()
            .expect("failed to  run git.");

        if !output.status.success() {
            panic!("failed to clone repository: {:#?}", output)
        }
    }

    for dir in directories {
        test_directory(out_path.clone(), out_path.join(dir), ignore);
    }
}

fn test_directory(root: PathBuf, directory: PathBuf, ignore: &[&str]) {
    let mut entries = fs::read_dir(&directory)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for entry in entries {
        if entry.is_dir() {
            test_directory(root.clone(), entry, ignore);

            continue;
        }

        if entry.is_file()
            && entry.extension().unwrap_or_default() == "php"
            && !ignore.contains(
                &entry
                    .as_path()
                    .strip_prefix(&root)
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )
        {
            let name_entry = entry.clone();
            let fullanme_string = name_entry.to_string_lossy();
            let name = fullanme_string
                .strip_prefix(root.to_str().unwrap())
                .unwrap();

            test_file(name, entry);
        }
    }
}

fn test_file(name: &str, filename: PathBuf) {
    let code = std::fs::read(&filename).unwrap();

    Lexer::new()
        .tokenize(&code)
        .map(|tokens| {
            Parser::new()
                .parse(tokens)
                .map(|_| {
                    println!("✅ successfully parsed file: `\"{}\"`.", name);
                })
                .unwrap_or_else(|error| {
                    panic!("❌ failed to parse file: `\"{name}\"`, error: {error:?}")
                })
        })
        .unwrap_or_else(|error| {
            panic!("❌ failed to tokenize file: `\"{name}\"`, error: {error:?}")
        });
}
