use full_moon::{
    ast,
    node::Node,
    print,
    tokenizer::{self, Token, TokenReference},
};
use insta::{assert_yaml_snapshot, glob};
use pretty_assertions::assert_eq;
use std::{
    borrow::Cow,
    fmt,
    fs::{self, File},
    io::Write,
    path::Path,
};

#[derive(PartialEq, Eq)]
struct PrettyString<'a>(pub &'a str);

// Make diff to display string as multi-line string
impl<'a> fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

fn unpack_token_reference<'a>(token: Cow<TokenReference<'a>>) -> Vec<Token<'a>> {
    token
        .leading_trivia()
        .chain(std::iter::once(token.token()))
        .chain(token.trailing_trivia())
        .cloned()
        .collect()
}

fn test_pass_case(path: &Path) {
    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(path);
    settings.remove_snapshot_suffix();
    settings.bind(|| {
        let source = fs::read_to_string(path.join("source.lua")).expect("couldn't read source.lua");

        let tokens = tokenizer::tokens(&source).expect("couldn't tokenize");

        assert_yaml_snapshot!("tokens", tokens);

        let ast = ast::Ast::from_tokens(tokens)
            .unwrap_or_else(|error| panic!("couldn't make ast for {:?} - {:?}", path, error));

        let old_positions: Vec<_> = ast.tokens().flat_map(unpack_token_reference).collect();
        let ast = ast.update_positions();
        assert_eq!(
            old_positions,
            ast.tokens()
                .flat_map(unpack_token_reference)
                .collect::<Vec<_>>(),
        );

        assert_yaml_snapshot!("ast", ast.nodes());
        assert_eq!(PrettyString(&print(&ast)), PrettyString(&source));
    })
}

fn test_pass_cases_folder<P: AsRef<Path>>(folder: P) {
    for entry in fs::read_dir(folder).expect("couldn't read directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        test_pass_case(&path)
    }
}

#[test]
#[cfg_attr(feature = "roblox", ignore)] // We don't want Roblox fields in JSON
#[cfg_attr(feature = "no-source-tests", ignore)]
fn test_pass_cases() {
    glob!("cases/pass/*", |path| test_pass_case(path));
}

#[test]
#[cfg(feature = "roblox")]
#[cfg_attr(feature = "no-source-tests", ignore)]
fn test_roblox_pass_cases() {
    test_pass_cases_folder("./tests/roblox_cases/pass");
}

#[test]
#[cfg(feature = "lua52")]
#[cfg_attr(feature = "no-source-tests", ignore)]
fn test_lua52_pass_cases() {
    test_pass_cases_folder("./tests/lua52_cases/pass");
}
