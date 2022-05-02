# Skel

make a project layout from a toml file

## Getting started

### basics
skel takes a project type and a name and will make the corresponding project in to that name

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

skel ether needs a global config pointing to project files (also called a skeletons), or a single project file. see [config](#config) for more.

### help
```
skel -- a project maker
make a project from a toml file

USAGE:
    skel [FLAGS] [OPTIONS] [ARGS]

ARGS:
    <TYPE>    a project type or alias to make
    <NAME>    the name of the project to make

FLAGS:
    -h, --help             Prints help information
    -n, --no-build         dont run the build script
    -N, --no-templating    dont make templates
    -V, --version          Prints version information

OPTIONS:
    -c, --different-config <FILE>    use FILE instead of the default config file
    -r, --different-root <PATH>      use PATH as root inserted of current dir
    -p, --project-file <FILE>        a project file to use instead of looking one up

```

### config


skel needs at least a skeleton project to make (or whats the point). these skeletons are toml files that have a few options like `files` and `dirs` to make as well as a `build` script that will be run with bash and a series of `templates` to add text. all strings including template files will be run through the template.

directory's and files will be made like you are using `mkdir -p` or `touch`

the `build` script will have `#!/usr/bin/env bash\n\n` appended to the top of the string

other then templating the `templates` (heh) they will act like you ran
```bash
cat path/to/main.js > /tmp/example_project/src/main.js
```


the variables:
  - dirs = the directory's to make
  - files = blank files to make
  - build = a string that becomes a build script
  - templates = a `path` to make in to
    and ether a `template` string or an `include` file to fill the given path

the slugs so far:
  - {{root}} = the root project directory (e.g. /tmp/example-project)
  - {{name}} = the new project name (e.g. cool-cli-tool)
  - {{config-dir}} = the config dir used this instance

example:
  - "{{root}}/src" = "/tmp/example-project/src"
  - "{{root}}/{{name}}/main.py" = "/tmp/example-project/example-project/main.py"


<br>
an example config file looks like

```toml
# ~/.config/skel/projects/javascript.toml

# you could skip `src` if you make `src/foo` (e.g. dirs = ["src/foo"])
dirs = [
    "src",
    "src/foo"
]

# this corresponds to the linux cmd `touch path/to/file`
# so if you have `src/foo/main.js` you need `src/foo` in the dirs list
files = [
    "src/main.js",
    "src/foo/{{name}}.txt"
]


# a build script that will be run by bash
# unless no files are to be made the script will be run in the project root
# if only the build variable is present the script will be run from the calling
# directory
build = """
# init the project
echo "$PWD"
echo "yarn init"

if [[ -f {{root}}/package.json ]]; then
    echo "yarn add -cwd '{{root}}' 'eslint'"
fi
"""

# the list of templates to be made
# these are added to the files list at runtime unless the --no-templating flag is present
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
# the template variable will be overridden by include but at least one is needed
[[templates]]
path  = ".eslint.json"
include = "{{config-dir}}/projects/basic-javascript/javascript.eslint"
```

you can call this file like

```bash
skel --project-file ~/.config/skel/projects/javascript.toml project-name
```


but skel can use a global config file that points to the project skeleton and defines  aliases to be use on the cli, the user config is located at `~/.config/skel/config.toml`


the config only has two options, `projects` and `aliases`. projects takes a key value pairs of a project skeleton names and a path to that skeleton. aliases takes a project name and a list of values to associate and become aliases for that project skeleton

an example config looks like

```toml
# ~/.config/skel/config.toml
# the paths to the projects
# {{config-dir}} will correspond to ~/.config/skel
[projects]
basic-javascript = "{{config-dir}}/projects/javascript.toml"
# the name or path dose not matter
new-python = "/path/to/python_project/python.toml"

# alias's to use on the cli
[aliases]
basic-javascript = ["js", "j"]
# these can be anything
new-python = ["py", "p", "this_is_not_shorter"]
```
