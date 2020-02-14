// TODO check if relative or absolute
// this is terrible
pub fn template(root: &str, name: &str, old_string: &str) -> String {
    let new_string: String = if old_string.contains("{{root}}") {
        old_string.replace("{{root}}", root)
    } else {
        old_string.to_string()
    };

    let new_string = if new_string.contains("{{name}}") {
        new_string.replace("{{name}}", name)
    } else {
        new_string
    };

    new_string
}
