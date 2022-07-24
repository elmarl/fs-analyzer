This app takes a path to a directory as input and gives you the top 5 largest files in the given directory. 

Results are given as the path to the file, its name and the size in bytes. Largest files first.

The number of files to list can be changed with the --num option.

Tested only on mac, may need changes for windows.

example command (Directly from vscode):

relative path
cargo run -- ..

absolute path
cargo run -- /Users/<user>/rust/fs-analyzer/

with num option
cargo run -- .. --num 1
