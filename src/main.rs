//! Command-line program for trying out the trees by hand.
//!
//! Run it with `cargo run --release`, then type commands such as
//! `insert 5`, `print` or `help`.

use std::io::{self, BufRead, Write};
use trees::{AvlTree, SearchTree};

const HELP: &str = "\
Commands:
  insert <number>   add a number (duplicates are ignored)
  delete <number>   remove a number
  search <number>   is the number in the tree?
  print             draw the tree
  inorder           list the numbers from smallest to largest
  height            height of the tree
  leaves            number of leaves
  empty             is the tree empty?
  help              show this list
  quit              leave
";

fn main() {
    let mut tree: AvlTree<i32> = AvlTree::new();
    println!("AVL tree. Type `help` to see the commands.");
    run(&mut tree);
}

/// Read commands until `quit` or the end of the input. Works with any tree
/// that implements `SearchTree`.
fn run(tree: &mut dyn SearchTree<i32>) {
    let stdin = io::stdin();
    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // end of input
            Ok(_) => {}
            Err(e) => {
                eprintln!("error: could not read input: {e}");
                break;
            }
        }

        let mut words = line.split_whitespace();
        let Some(command) = words.next() else { continue };
        let argument = words.next();

        match command {
            "insert" => {
                if let Some(key) = number(command, argument) {
                    if tree.insert(key) {
                        println!("inserted {key}");
                    } else {
                        println!("{key} is already in the tree, nothing added");
                    }
                }
            }
            "delete" => {
                if let Some(key) = number(command, argument) {
                    if tree.delete(&key) {
                        println!("deleted {key}");
                    } else {
                        println!("{key} is not in the tree");
                    }
                }
            }
            "search" => {
                if let Some(key) = number(command, argument) {
                    let answer = if tree.contains(&key) { "found" } else { "not found" };
                    println!("{key}: {answer}");
                }
            }
            "print" => tree.print_tree(),
            "inorder" => tree.print_in_order(),
            "height" => println!("height: {}", tree.height()),
            "leaves" => println!("leaves: {}", tree.count_leaves()),
            "empty" => println!("{}", if tree.is_empty() { "yes, empty" } else { "no" }),
            "help" => print!("{HELP}"),
            "quit" | "exit" => break,
            other => println!("unknown command `{other}`. Type `help` for the list."),
        }
    }
}

/// Turn the word after a command into a number, or explain what is wrong.
fn number(command: &str, argument: Option<&str>) -> Option<i32> {
    match argument {
        None => {
            println!("`{command}` needs a number, for example `{command} 5`");
            None
        }
        Some(text) => match text.parse::<i32>() {
            Ok(n) => Some(n),
            Err(_) => {
                println!("`{text}` is not a whole number");
                None
            }
        },
    }
}
