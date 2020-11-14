# todo
## short term
- pass unused args to the build script
- make all string path joins use dynamic path char
- add the location / why the errors happened, like is it a template file that
  cant be found

## long term
- make errors better
- decide how flexible / intricate this should be
    * for one should you be able to make files out side the project dir
    * should i bother with fake root or something for the build script
    * maybe come up with a way to point to a directory to recreate
    * maybe use a templating library
- make auto complete better
- maybe add include for the build script
- allow hiding stderr
- refactor defaults to be more simple

## test
- make test for optional config variables
- make test for cli interface
- test for failures
    * there are templates that aren't in the file list
