// these fucking suck
pub fn template(root: &str, name: &str, old_string: &str) -> String {
    let new_string = old_string.replace("{{root}}", root);

    new_string.replace("{{name}}", name)
}

pub fn template_str(replace_with: &str, old_string: &str) -> String {
    old_string.replace("{{config-dir}}", replace_with)
}
