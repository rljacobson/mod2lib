mkdir mod2lib
cd mod2lib

# Initialize a new Cargo library project
cargo init --lib

# Create the main directories and files
mkdir benches examples tests resources doc

# Create the binary example
cargo new --bin examples/basic_example

# Create benchmark files
touch benches/benchmark.rs

touch README.md
curl https://opensource.org/licenses/MIT > LICENSE-MIT
curl https://www.apache.org/licenses/LICENSE-2.0.txt > LICENSE-APACHE
touch .gitignore

echo "# Standard Rust project .gitignore" > .gitignore
curl https://raw.githubusercontent.com/github/gitignore/main/Rust.gitignore >> .gitignore

# Add the doc directory
touch doc/.gitkeep

# Update Cargo.toml to add benches and example binary
cat <<EOF >> Cargo.toml

[[bench]]
name = "benchmark"
path = "benches/benchmark.rs"

[[example]]
name = "basic_example"
path = "examples/basic_example/src/main.rs"
EOF
