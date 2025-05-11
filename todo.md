## TODO - base

Redo `util`
- add `use` subcommand
  - chooses language, platform(s)
- `configure` will choose tools
  - tools will not be chosen in the `build` command
- add `reconfigure` command
  - used if project is moved or tools are changed
- path to each tool will be in config file
- change script structure
  - run scripts in `files/base/scripts` instead of one bloated script
  - each lang and arch will have its own `build` script


# util
use .ini config format  
config file for each architecture:  
- build-<arch>.conf
```
  file: util-x86_64.conf
  x86_64_compiler=x86_64-elf-gcc
  x86_64_linker=x86_64-elf-ld
```

```
  file: util-arm64.conf
  arm64_compiler=aarch64-elf-gcc
  arm64_linker=aarch64-elf-gcc
  ...
```