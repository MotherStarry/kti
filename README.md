# kti
A simple CLI tool to correct file extensions to match their file signatures.

### What kti does:
kti scans recursively through directories and finds files that have file extensions that do not match their file signature.

### How to install kti:
You first need a working installation of the Rust compiler. Simply visit [rustup](https://rustup.rs) and follow the steps for the operating system you are using.
After you are done and have a working version of rust, you can run this command:
```fish
cargo install --git https://github.com/MotherStarry/kti
```
Do make sure to add `.cargo/bin` to your path.

### Usage:
I strongly recommend you first read through the couple options kti has available, to do so just type ``kti -h`` or ``kti --help`` in your terminal.

**Important**: By default kti will change all file extensions for the files that do not match with what kti has found. I advise you first run kti with the --dry-run option and verifying the changes kti plans to make before running kti without it. Example:
```fish
kti -d --dry-run
```
This command will show you what files kti *would* change.

If you only want to use kti on a single file you can do so with:
```fish
kti your_file.png
```

if you just want to recursively correct all files in the directory you are in its as simple as:
```fish
kti
```
running this will not add any color to the output and will print out all files whether they have different extensions or not and it will also skip hidden files.

if you wish to silence kti you can add the -s option like here:
```fish
kti -s
```

I suggest using kti with the -d option and -c for prettier and likely more readable output or if you wish to limit your search depth you can use the -m option.

