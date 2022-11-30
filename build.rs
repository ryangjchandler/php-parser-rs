use std::env;
use std::fs::read_dir;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=tests");

    if env::var("BUILD_INTEGRATION_TESTS").unwrap_or_else(|_| "0".to_string()) == "0" {
        return;
    }

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut entries = read_dir(manifest.join("tests"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    let mut content = String::new();
    content.push_str("/// this file is auto-generated by the build script.\n");
    content.push_str("/// you should never manually change it.\n\n\n");

    for entry in entries {
        let code_filename = entry.join("code.php");
        let ast_filename = entry.join("ast.txt");
        let lexer_error_filename = entry.join("lexer-error.txt");
        let parser_error_filename = entry.join("parser-error.txt");

        if !code_filename.exists() {
            continue;
        }

        if ast_filename.exists() {
            assert!(
                !lexer_error_filename.exists(),
                "`lexer-error.txt` was not expected for `{}`.",
                entry.to_string_lossy()
            );
            assert!(
                !parser_error_filename.exists(),
                "`parser-error.txt` was not expected for `{}`.",
                entry.to_string_lossy()
            );

            content.push_str(&build_success_test(entry, code_filename, ast_filename))
        } else if lexer_error_filename.exists() {
            assert!(
                !parser_error_filename.exists(),
                "`parser-error.txt` was not expected for `{}`.",
                entry.to_string_lossy()
            );

            content.push_str(&build_lexer_error_test(
                entry,
                code_filename,
                lexer_error_filename,
            ))
        } else {
            assert!(
                parser_error_filename.exists(),
                "unable to find `parser-error.txt` for `{}`.",
                entry.to_string_lossy()
            );

            content.push_str(&build_parser_error_test(
                entry,
                code_filename,
                parser_error_filename,
            ))
        }
    }

    let dest = manifest.join("tests").join("integration_test.rs");
    std::fs::write(dest, content).expect("failed to write to file");
}

fn build_success_test(entry: PathBuf, code_filename: PathBuf, ast_filename: PathBuf) -> String {
    format!(
        r#"#[test]
fn test_success_{}() {{
    use php_parser_rs::{{Lexer, Parser}};
    use pretty_assertions::assert_str_eq;

    let code_filename = "{}";
    let ast_filename = "{}";

    let code = std::fs::read_to_string(&code_filename).unwrap();
    let expected_ast = std::fs::read_to_string(&ast_filename).unwrap();

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(code.as_bytes()).unwrap();
    let mut parser = Parser::new(None);
    let ast = parser.parse(tokens).unwrap();

    assert_str_eq!(expected_ast.trim(), format!("{{:#?}}", ast));
}}

"#,
        entry.file_name().unwrap().to_string_lossy(),
        code_filename.to_string_lossy(),
        ast_filename.to_string_lossy()
    )
}

fn build_lexer_error_test(
    entry: PathBuf,
    code_filename: PathBuf,
    lexer_error_filename: PathBuf,
) -> String {
    format!(
        r#"#[test]
fn test_lexer_error_{}() {{
    use php_parser_rs::Lexer;
    use pretty_assertions::assert_str_eq;

    let code_filename = "{}";
    let lexer_error_filename = "{}";

    let code = std::fs::read_to_string(&code_filename).unwrap();
    let expected_error = std::fs::read_to_string(&lexer_error_filename).unwrap();

    let mut lexer = Lexer::new(None);
    let error = lexer.tokenize(code.as_bytes()).err().unwrap();

    assert_str_eq!(expected_error.trim(), format!("{{:?}}", error));
}}

"#,
        entry.file_name().unwrap().to_string_lossy(),
        code_filename.to_string_lossy(),
        lexer_error_filename.to_string_lossy()
    )
}

fn build_parser_error_test(
    entry: PathBuf,
    code_filename: PathBuf,
    parser_error_filename: PathBuf,
) -> String {
    format!(
        r#"#[test]
fn test_paser_error_{}() {{
    use php_parser_rs::{{Lexer, Parser}};
    use pretty_assertions::assert_str_eq;

    let code_filename = "{}";
    let parser_error_filename = "{}";

    let code = std::fs::read_to_string(&code_filename).unwrap();
    let expected_error = std::fs::read_to_string(&parser_error_filename).unwrap();

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(code.as_bytes()).unwrap();

    let mut parser = Parser::new(None);
    let error = parser.parse(tokens).err().unwrap();

    assert_str_eq!(
        expected_error.trim(),
        format!("{{:?}} -> {{}}", error, error.to_string()),
    );
}}

"#,
        entry.file_name().unwrap().to_string_lossy(),
        code_filename.to_string_lossy(),
        parser_error_filename.to_string_lossy()
    )
}
