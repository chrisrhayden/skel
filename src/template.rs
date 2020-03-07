// these fucking suck
pub fn template(root: &str, name: &str, old_string: &str) -> String {
    let new_string = old_string.replace("{{root}}", root);

    new_string.replace("{{name}}", name)
}

pub fn template_str(user_str: &str, old_string: &str) -> String {
    old_string.replace("{{config-new}}", user_str)
}
