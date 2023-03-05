# Skel

make a project layout from a toml file

## Getting started

### basics
skel takes a project type and a name and will make the corresponding project in
to that name

```bash
skel javascript project-name

tree project-name
#  project-name/
#  ├── node-modules [116 entries exceeds filelimit, not opening dir]
#  ├── package.json
#  ├── src
#  │   └── main.js
#  └── yarn.lock
```

skel can also use alias's like so
```bash
skel js name

skel j name
```

skel ether needs a global config pointing to project files (also called
skeletons), or a single project file. see [config](#config) for more.

### help
```
skel
make a project from a skeleton defined in a toml file

USAGE:
    skel [OPTIONS] [ARGS]

ARGS:
    <SKELETON>    the skeleton to make, can be the skeleton name or alias
    <NAME>        the name of the new project to make

OPTIONS:
    -a, --alt-config-path <ALT_CONFIG_PATH>    a path to a main config file
    -d, --dry-run                              print out what will be done
    -D, --different-root <DIFFERENT_ROOT>      a different root to make the project in to
    -h, --help                                 Print help information
    -s, --skeleton-file <SKELETON_FILE>        a path to a skeleton file
```

### config


skel needs at least a skeleton project to make (or whats the point). these
skeletons are toml files that have a few options like `files` and `dirs` to make
as well as a `build` script that will be run with bash and a series of
`templates` to add text. all files will can be templated with handle bar syntax.

all paths will have the project root added in front so you just need to define
paths as if they will be made in the project root

the `build` script will have `#!/usr/bin/env bash\n\n` appended to the top of
the string

the skeleton variables:
  - dirs = list of strings: the directory's to make
  - files = list of strings: blank files to make
  - build = string: a string that becomes a build script
  - build_first = bool: if the build script should be run first
  - templates = object {path: string, template: string, include: string}
      - path: the path in the new project that the template should be made to
      - template: the text that should be written to the new file
      - include: a path to a file whose contents should be copied to the new file

the templating slugs:
  - {{root}} = the root project (e.g. /tmp/cool-cli-tool)
  - {{name}} = the new project name (e.g. cool-cli-tool)
  - {{config-dir}} = the config dir used this instance
  - {{env "ENV_VAR"}} = use an env variable

example:
  - "cd {{root}}" -> "cd /tmp/cool-cli-tool"
  - "{{root}}/{{name}}/main.py" -> "/tmp/cool-cli-tool/cool-cli-tool/main.py"
  - "{{config-dir}}/project/bash.toml" -> "/home/user/.config/skel/project/bash.toml"


an example skeleton file looks like

```toml
# you could skip `src` if you make `src/foo`
dirs = [
    "src/foo"
]

# the parent dirs will be made for all files so if you have `src/bar/file.txt`
# then you do not need to add `src/bar` to the `dirs` list
files = [
    "src/main.js",
    "src/bar/{{name}}.txt"
]

# a build script that will be run with bash
build = """
# init the project
echo "$PWD"
echo "yarn init"

if [[ -f {{root}}/package.json ]]; then
    echo "yarn add -cwd '{{root}}' 'eslint'"
fi
"""

# if the build script should be run first
#
# this is useful if you are using another project maker like `cargo new` and
# want to add things after it is run
#
# this is optional and will default to false
build_first = false

# the list of templates to be made
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

# you can also include another file by using the include variable,
# include file will also be templated
[[templates]]
path  = ".eslint.json"
include = "{{config-dir}}/projects/basic-javascript/javascript.eslint"
```

you can call this file like

```bash
skel --skeleton-file ~/.config/skel/projects/javascript.toml project-name
```

the parent directory for the skeleton file will be used for the `{{config-dir}}`
slug when templating, for the example above the `{{config-dir}}` slug will
become `/home/user/.config/skel/projects`


but skel can use a global config file that points to the project skeleton and
defines  aliases to be use on the cli, the user config is located at
`$XDG_CONFIG_HOME/skel/config.toml`

an example config looks like

```toml
[skeletons]
# the path or aliases dose not matter
basic_javascript.path = "{{config-dir}}/projects/javascript.toml"
basic_javascript.aliases = ["js", "j"]
new-python.path = "/path/to/python_project/python.toml"
new-python.aliases = ["py", "p", "this_is_not_shorter"]
```

## TODO

- update the build script logic
  - the program should cd in to the containing dir for the project
- allow this to be used without a main config
