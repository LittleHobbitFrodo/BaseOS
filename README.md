# BaseOS 
BaseOS is a template for your operating system. It contains several libraries, including a minimized strandard library (ministd) and a set of tools for compiling the system, installing dependencies, etc.

# IMPORTANT
BaseOS is not yet finished and may not work. It still needs tinkering...

# Main Goal
The main goal of this project is to create a reliable and simple platform for creating multi-platform operating systems for various 64 bit platforms. So far only the x86_64 target is supported.

# What is provided
BaseOS provides simplified standard library for Rust and tools that take care of dependencies, building and running the operating system.  
This project is not intended to be an example of an operating system, only its foundation.

# Licence
The whole project is under the **M.I.T.** license, which allows you to add code/files to the project under different licenses (even proprietary).

# Tools
Tools are available with the `util` tool. It performs project initialization and administration, builds the operating system and installs/removes development dependencies.
It also offers the `./util help` command to learn more.

# Requirements
A UNIX-based operating system is required to use the project. You can use Linux, BSD or MacOS. These systems should be fully supported:
- [X] MacOS (with Homebrew)
- [X] Fedora Linux (tested on Asahi and workstation)
- [ ] Arch Linux (pacman + yay?)
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
- [X] Rust (60% of the work is done)

# Development Dependencies
### Rust
- **rustup** (regardless of the target platform)
  - The `util` utility will install and set it up for you

### Other Required dependencies
- **Xorriso** - builds the ISO

### Optional dependencies
- **Qemu** - emulator to test the OS on

# Lets get started!
1. To initialize the project, simply clone the base branch
    - `git clone https://github.com/LittleHobbitFrodo/BaseOS.git`
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
  - [X] Project Info
  - [X] Project configuration
  - [X] Project Reconfiguration
  - [X] Running
  - [ ] Dependency management (not all system are supported at the moment)
  - [X] Project management
  - [X] mkiso
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
  - [X] Panicking
  - [X] Heap
    - buddy_system_allocator
  - [X] String (untested)
  - [X] Box
    - `Array` is used for allocation of arrays
  - [X] Vector
  - [ ] HashMap
  - [ ] Rc
  - [ ] Arc
