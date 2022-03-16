#[derive(Debug)]
pub struct TemplateArgs<'a> {
    pub project_name: &'a str,
    pub project_root_path: &'a str,
    pub skel_config_path: &'a str,
}

// these fucking suck
pub fn template(template_view: &TemplateArgs, old_string: &str) -> String {
    let new_string =
        old_string.replace("{{root}}", template_view.project_root_path);

    let new_string =
        new_string.replace("{{config-dir}}", template_view.skel_config_path);

    new_string.replace("{{name}}", template_view.project_name)
}
