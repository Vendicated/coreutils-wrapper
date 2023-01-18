# coreutils wrapper

A wrapper around [Rust coreutils](https://github.com/uutils/coreutils) (cross platform rewrite of GNU coreutils)
that creates shims for each command so you can call them directly (`ls -la` instead of `coreutils ls -la`)

Should technically work on any platform, but this is only really useful on Windows, since there's not really any
reason to use this over actual GNU coreutils

GNU coreutils actually have a [native port](https://gnuwin32.sourceforge.net/packages/coreutils.htm) but they're
very slow from my experience

The way it works is that it creates a symlink to itself for each coreutils command in the current directory.
You can then (assuming the directory you linked to is in your PATH) just run the command directly.

If you're using powershell, you will likely automatically have aliases for commands like ls to inferior powershell
versions, run this program with the --pwsh flag to fix the problem

## Usage

Make sure your cargo bin is in the PATH

You must first install coreutils: `cargo install coreutils`

Clone and install this program with `cargo install --path .`

Now you can install the shims to the current directory by running `coreutils-wrapper --link`
(run this in ~/.local/bin for example, any directory in PATH will work)

```
$ coreutils-wrapper --help
  ================================================
                  coreutil-wrapper
  ================================================

  This is a wrapper for the rust coreutils command
  that allows you to run commands directly: `sort`
  instead of `coreutils sort`. Arguments and stdin
  are of course forwarded to the desired command!!

  You should usually not call me directly, instead
  run me with the --link flag to create a shim for
  each command in the current working directory :3
  Make sure the directory is in your PATH, or else
  you won't actually be able to call the commands!

  Usage:
     --help: Show this help message and exit (^_^)
     --link: Create a shim for each command in the
             current directory. This will override
             any existing files with the same name
     --drop: Delete shims in the current directory
     --pwsh: Help with removing powershell aliases

```
