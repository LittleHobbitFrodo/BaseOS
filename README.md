# BaseOS 
BaseOS is a template for your operating system project. It performs all the activities around it, such as building the kernel and creating a disk image.

# IMPORTANT
BaseOS is not yet finished and may not work. It still needs tinkering...

# Main Goal
The main goal of this project is to create a reliable and simple platform for creating multi-platform operating systems in several programming languages and on different 64 bit platforms. So far only the x86_64 target is supported.

# What is provided
BaseOS provides simplified standard libraries for each language and tools that take care of dependencies, building and running the operating system.  
This project is not intended to be an example of an operating system, only its foundation.

# Licence
The whole project is under the **M.I.T.** license, which allows you to add code/files to the project under different licenses (even proprietary).

# Tools
Tools are available with the `util` tool. It performs project initialization and administration, builds the operating system and installs/removes development dependencies.
It also offers the `./util help` command to learn more.

# Requirements
A UNIX-based operating system is required to use the project. You can use Linux, BSD or MacOS. These systems should be fully supported:
- [X] MacOS (with Homebrew)
- [ ] Fedora Linux
- [ ] Arch Linux (with yay)
- [ ] Ubuntu
- [ ] Debian
- [ ] CentOS
- [ ] FreeBSD
- [ ] OpenBSD

Please note that some development **dependencies** must be **built from source** on certain platforms. If your operating system is not listed above, it does not mean that it is not supported.

# Supported Target Platforms
- [X] x86_64
- [ ] arm64

# Supported Programming Languages
- [X] Rust (75% of the work is done)
- [X] C (±25% => The stdlib has not been worked on yet)

# Development Dependencies
### C
- x86_64
  - **x86_64-elf-gcc** - Or similar compiler
- arm64
  - **aarch64-elf-gcc** - Or similar
### Rust
- **rustup** (regardless of the target platform)
  - The `util` utility will install and set it up for you

### Other Required dependencies
- **Xorriso** - builds the ISO

### Optional dependencies
- **Qemu** - emulator to test the OS on

# Lets get started!
1. To initialize the project, simply clone the base branch
    - `git clone https://github.com/LittleHobbitFrodo/baseOS.git --branch base`
2. Choose your target platform and programming language
3. Configuring the project using the `util` tool
    - `./util conf <arch> <lang>`
      - for example `./util conf x86_64 rust`
4. You can also download development dependencies
    - `./util dep install`
    - dependencies can be removed using `./util dep remove`


# Roadmap
### Base
- Util
  - [ ] Project Info
  - [X] Project configuration
  - [X] Project Reconfiguration
  - [X] Running
  - [ ] Dependency management
  - [X] Project management
  - [X] mkiso
  
- TODO
  - when changing kernel name limine configs must be updated
    - once done update mkiso to work with it
  - add assembler support
    - assembler for each architecture
    - specified as parameter for conf subcommand?
  - instead of using paths to linker, compiler etc, the path will be statisally pointing to symlinks pointing to the tools
  - add compiler_parameters.conf
### C
- [X] Bootable
- [X] Renderer
- [ ] STDlib

### Rust
- [X] Bootable
- [X] Renderer
- [ ] STDlib
  - [X] Bootloader communication
    - limine-rs crate
  - [X] Text Rendering
    - [X] Output Formatting
  - [X] Sync primitives
    - used Spin crate
  - [ ] Panicking
  - [X] Heap
    - buddy_system_allocator
  - [X] String (untested)
  - [ ] Box
  - [ ] Vector
  - [ ] HashMap
  - [ ] Rc
  - [ ] Arc
