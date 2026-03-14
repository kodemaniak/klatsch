default:
  just --list

maelstrom test binary node-count time-limit *args='':
    cargo build
    $HOME/Development/tools/maelstrom/maelstrom test \
        -w {{ test }} \
        --bin ~/Development/klatsch/target/debug/{{ binary }} \
        --node-count {{ node-count }} \
        --time-limit {{ time-limit }} \
        {{ args }}
