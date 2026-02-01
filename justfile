default:
  just --list

maelstrom test binary:
    cargo build
    $HOME/Development/tools/maelstrom/maelstrom test -w {{ test }} --bin ~/Development/klatsch/target/debug/{{ binary }} -- --node-count 1 --time-limit 10
