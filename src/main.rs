// coreutils-wrapper
// Copyright (C) 2023  Vendicated
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    env::{self, args, current_dir},
    fs,
    io::{stdin, stdout, ErrorKind, Write},
    path::Path,
    process::{exit, Command, Stdio},
};

use symlink::symlink_file;

fn get_coreutils_commands() -> Vec<String> {
    let stdout = Command::new("coreutils").output().unwrap().stdout;
    let output =
        String::from_utf8(stdout).expect("Failed to parse output of coreutils. Invalid utf-8");

    // I don't wanna add regex crate leave me alone
    let list_idx = output
        .find("Currently defined functions")
        .expect("Failed to parse coreutils command list");
    let start_idx = list_idx
        + output[list_idx..]
            .find("[,")
            .expect("Failed to parse coreutils command list");
    let end_idx = list_idx
        + output[list_idx..]
            .find("yes")
            .expect("Failed to parse coreutils command list");

    return output[start_idx..end_idx + 3]
        .split(',')
        .map(|s| s.trim().to_owned())
        .collect();
}

fn prompt_ok() {
    print!("Are you sure you want to continue? [y/N] ");
    stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() != "y" {
        println!("Aborting");
        exit(1);
    }
}

fn do_link() -> ! {
    let mut files_to_create = get_coreutils_commands();
    if cfg!(windows) {
        for file in &mut files_to_create {
            *file += ".exe"
        }
    }

    println!(
        "The following symlinks will be created in {}:\n\n{}\n",
        current_dir().expect("Failed to get cwd").display(),
        files_to_create.join(", ")
    );
    let existing_files_count = files_to_create
        .iter()
        .filter(|f| Path::new(f).exists())
        .count();

    match existing_files_count {
        1 => println!(
            "WARNING: {} file already exists and will be overwritten!",
            existing_files_count
        ),
        0 => {}
        _ => println!(
            "WARNING: {} of those files already exist and will be overwritten!",
            existing_files_count
        ),
    }

    prompt_ok();

    let cwd = current_dir().expect("Failed to get cwd");
    let own_path = cwd.join(args().next().expect("explosion"));

    for file in &files_to_create {
        if let Err(e) = fs::remove_file(file) {
            if e.kind() != ErrorKind::NotFound {
                eprintln!("Failed to delete existing file {}: {}", file, e);
                continue;
            }
        }

        if let Err(e) = symlink_file(&own_path, file) {
            eprintln!("Failed to create symlink {}: {}", file, e);
        }
    }

    println!(
        "Done! Successfully created {} symlinks!",
        files_to_create.len()
    );

    let path_var = &env::var("PATH").expect("Failed to get PATH");
    let is_cwd_in_path = env::split_paths(&path_var).any(|p| p == cwd);
    if !is_cwd_in_path {
        println!(
            "Warning: {} is not in the PATH environment variable. You must add it to PATH for the shims to work.",
            cwd.display()
        );

        if !cfg!(windows) {
            println!(
                "Add the following to your .bashrc/.zshrc/...:\n    export PATH=\"{}:$PATH\"",
                cwd.display()
            );
        }
    }

    if cfg!(windows) {
        println!("Hint: Powershell creates a lot of aliases for coreutils commands");
        println!("which will take precedence over the shims. Run me with the --pwsh");
        println!("flag for info on how to fix this.");
    }

    exit(0);
}

fn do_drop() -> ! {
    let cwd = current_dir().expect("Failed to get cwd");
    let commands = get_coreutils_commands();
    let files_to_delete = commands
        .iter()
        .filter_map(|f| {
            let mut path = cwd.join(f);
            if cfg!(windows) {
                path.set_extension("exe");
            }
            if path.exists() {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if files_to_delete.is_empty() {
        println!("No shims to drop!");
    } else {
        println!(
            "The following files will be deleted:\n\n{}\n",
            files_to_delete
                .iter()
                .map(|p| p
                    .file_name()
                    .expect("No filename HOW")
                    .to_str()
                    .expect("non utf-8 filename HOW")
                    .to_owned())
                .collect::<Vec<_>>()
                .join(", ")
        );
        prompt_ok();

        let mut all_ok = true;
        for file in &files_to_delete {
            if let Err(e) = std::fs::remove_file(file) {
                eprintln!("Failed to delete {}: {}", file.display(), e);
                all_ok = false;
            }
        }
        if all_ok {
            println!(
                "Done! Successfully deleted {} files!",
                files_to_delete.len()
            );
        } else {
            println!("Done! Some files failed to delete :(");
        }
    }

    exit(0);
}

// not only on windows because some people may be insane and use pwsh on linux
fn do_pwsh() -> ! {
    println!("To remove aliases for all coreutils from powershell:");
    println!("    - Open powershell and type $profile to find your powershell config");
    println!("    - Open the file with any text editor (create it if it doesn't exist)");
    println!("    - Paste the following and restart powershell:");

    let cmd_list = get_coreutils_commands()
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<_>>()
        .join(", ");

    let delim = "#====================#";
    let cmd = include_str!("./powershell_fix.ps1")
        .trim()
        .replace("%CMD_LIST_PLACEHOLDER%", &cmd_list);

    println!("{0}\n{1}\n{0}", delim, cmd);

    exit(0);
}

fn main() {
    let args = args().collect::<Vec<String>>();

    let utility = Path::new(&args[0])
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    if utility == "coreutils-wrapper" {
        match args.get(1).map(|s| s.as_str()) {
            Some("-link" | "--link") => do_link(),
            Some("-drop" | "--drop") => do_drop(),
            Some("--pwsh") => do_pwsh(),
            _ => {
                // I put so much effort into making every line the same width
                // istg if some asshole runs it with a non monospace font I'm
                // gonna be sosososooooooooooooooooooooooooooooooooo pissed!!
                println!(include_str!("./help.txt"));
                exit(0);
            }
        }
    }

    let mut argv = args[1..].to_vec();
    argv.insert(0, utility);

    match Command::new("coreutils")
        .args(&argv)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()
    {
        Ok(status) => exit(status.code().unwrap_or(1)),
        Err(err) => eprintln!("Failed to execute coreutils with args {:?}: {}", argv, err),
    }
}
