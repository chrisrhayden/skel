// this is terrible
pub fn template(root: &str, name: &str, old_string: &str) -> String {
    let new_string = old_string.replace("{{root}}", root);

    new_string.replace("{{name}}", name)
}
