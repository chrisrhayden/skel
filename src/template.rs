// these fucking suck
pub fn template(
    root: &str,
    name: &str,
    config: &str,
    old_string: &str,
) -> String {
    let new_string = old_string.replace("{{root}}", root);

    let new_string = new_string.replace("{{config-dir}}", config);

    new_string.replace("{{name}}", name)
}
