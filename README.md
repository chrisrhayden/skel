# Skel

make a project layout from a toml file

## Getting started
skel takes a project type and a name and will make the project in to that name
```bash
skel cp name
```

the user config looks like

```toml
# the paths to the projects
# {{config-dir}} will correspond to ~/.config/skel
[projects]
basic_cpp        = "{{config-dir}}/projects/cpp.toml"
basic_javascript = "{{config-dir}}/projects/javascript.toml"
basic_python     = "{{config-dir}}/projects/python.toml"

# alias's to use on the cli
[alias]
basic_cpp        = ["cpp", "cp", "c++"]
basic_javascript = ["js"]
basic_python     = ["py", "p"]
```

and a project config looks like

```toml
# all file's/directory's will be made to the root level
#   `src/foo` will turn in to /current/path/test_js_project/src/foo

# all strings will be run through a template
# the slugs so far are:
#     - {{root}} = the root project directory (e.g. /tmp/example_project)
#     - {{name}} = the new project name (e.g. cool_cli_tool)
#     - {{config-dir}} = the config dir used this instance
#
# example:
#   src = "{{root}}/src" = "/tmp/example_project/src"
#   main = "{{root}}/src/main.js" = "/tmp/example_project/src/main.rs"

# make all directory's listed
# this wont fail on already made dirs,
# so having the same dirs is fine (e.g. dirs = ["src", "src"])
# each dir will correspond to the linux cmd `mkdir -p path/to/dir`
# so you could skip `src` if you make `src/foo` (e.g. dirs = ["src/foo"])
dirs = [
    "src",
    "src/foo"
]

# this corresponds to the linux cmd `touch path/to/file`
# so if you have `src/foo/main.js` you need `src/foo` in the dirs list
files = [
    "src/main.js",
    "src/foo/{{name}}.txt"
    ".eslint.json",
]


# a build script that will be run by bash
# unless no files are to be made the script will be run in the project root
# if only the build variable is present the script will be run from the calling
# directory
# `#!/usr/bin/env bash` will be added, probably a bad idea
build = """
# init the project
echo "$PWD"
echo "yarn init"

if [[ -f {{root}}/package.json ]]; then
    echo "yarn add -cwd '{{root}}' 'eslint'"
fi
"""

# basic templates to be made, the same slugs apply
# these are added to the files to be made no mater what
[[templates]]
path = "src/main.js"
template = """function run() {
    console.log("hello {{name}}");
    return true;
}

function main() {
    run();
}

main();

module.run = run;
"""

# you can also include another file by giving it a name, you can also template
# the include path the template variable will be overridden by the include
# variable but one is needed
[[templates]]
path  = ".eslint.json"
# use a file a ~/.config/skel/projects/javascript.eslint
# instead of a template string
include = "{{config-dir}}/projects/basic_javascript/javascript.eslint"
```
