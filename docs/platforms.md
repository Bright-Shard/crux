# Platforms Supported by Crux

Let's define what a "platform" is first. A platform is a processor type (e.g. x86, x86_64, ARM, RISC-V), operating system (e.g. Windows, Linux), and file type (e.g. DLL, EXE, ELF).

These three components are basically the three things that change how a program runs on a computer. The processor changes assembly instructions; the operating system changes what functions and APIs code can use; and the file type changes how code gets loaded in memory. So, these are the three variables that Crux has to support to run on a specific machine - Crux needs to support the machine's processor, the machine's operating system, and the machine's file format.

Currently, Crux only works with Unix operating systems in the ELF or SO file formats, on any processor supported by LLVM. In practical terms, Crux programs can only be compiled to executables and dynamic libraries for Linux-like operating systems (not Unix operating systems; not all Unix operating systems use the ELF/SO file formats, for example macOS uses the Mach-O format).

Generally speaking, Crux should run on any processor supported by LLVM, since it doesn't rely on any handwritten assembly and Rust can be compiled with LLVM. I'd like to avoid handwritten assembly as much as possible in Crux to ensure it remains this portable - it just makes supporting new platforms easier.

Crux doesn't support all operating systems, since operating systems have different APIs for things like memory allocations. Crux also doesn't support all file formats because different file formats change things like the 'entrypoint' of the file that gets executed when the file is loaded in memory.





# Porting Crux to New Platforms



## New Operating Systems

Different operating systems use different APIs to accomplish similar (or the same) goal. For example, virtual memory allocations use `mmap` on Unix but `VirtualAlloc` on Windows. Therefore porting Crux to new operating systems mostly just involves finding the new operating system's version of a particular function and calling it over FFI where necessary.

All Crux functions that involve calling OS APIs have this code at the bottom of the function:

```rs
#[cfg(not(supported_os))]
compile_error!("unimplemented on this operating system");
```

This basically gives you a to-do list when compiling for a new operating system. Just find all the compilation errors and replace them with calls to the correct OS API. Then you can add the new OS to the `supported_os` cfg alias in `build.rs`, which will get rid of the `compile_error`s. Finally, run all of Crux's unit tests to make sure everything works as intended.



## New File Formats

Different file types get loaded in different ways by the operating system. As an example, Windows will use the `DllMain` function as the entry point for a `.dll` file, but uses the `mainCRTStartup` (or a few others... it's complicated) function as the entry point for a `.exe` file. Crux has to load some platform-specific data when the program starts (see `crux::rt::startup_hook`), so it must control the program's entry point.

So, the first challenge of supporting a new file format is to make an entrypoint for that format. Generally your platform-specific entrypoint should call the standard Crux entrypoint (`crux::rt::entrypoint`), and the various platform-specific entrypoints live in `src/rt/os/entrypoint`.

Different file formats *also* use different ways to store data. Generally speaking, executable files are always broken down into "sections", which have different properties (e.g. there may be one section Rust uses to store compile-time constants, which the OS will then load into read-only memory when your app gets launched). Crux stores custom data in the file the Crux program compiles to (for example, ini functions are usually stored in the `.crux.ini` section), so Crux has to have some control over the output of the file to make sure it can access the data it stores at compile-time.

Getting this part working is probably the trickiest part. You'll probably have to write [more linker scripts](../link-scripts) or change the linker arguments set by [`crux-build`](../crates/crux-build) to ensure everything works. You may also have to change statics Crux reads from.
