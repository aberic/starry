# cargo-audit - Audit Cargo.lock for crates with security vulnerabilities
# cargo-asm, cargo-llvm-ir - Shows generates assembly or LLVM IR of Rust code
# cargo-benchcmp - Compare output of cargo bench output, both runs over time and same benchmarks in multiple modules (e.g. for comparing multiple implementations)
# cargo-bitbake - Generate Yocto's bitbake recipes from your Cargo.toml
# cargo-bloat - Find out what takes most of the space in your executable.
# cargo-cache - Helps you manage the cargo cache (~/.cargo), print sizes and clear directories
# cargo-check - This is a wrapper around cargo rustc -- -Zno-trans. It can be helpful for running a faster compile if you only need correctness checks.
# cargo-cook - Cooks your crate (packaging & deploying).
# clippy - Lint your project using Clippy.
# cargo-cln - Alternative to cargo-clean, allows running arbitrary commands in addition to wiping out target/ directory.
# cargo-clone - Fetch source code of a crate
# cargo-config - Print info about the current crate.
# cargo-count - counts lines of code in cargo projects, including giving naive unsafe statistics
# cargo-deadlinks - Check your cargo doc documentation for broken links
# cargo-do - Run multiple cargo subcommands in sequence (e.g., cargo do clean, build)
# cargo-deb - Generates & builds Debian packages from cargo projects.
# cargo-deps - Create dependency diagrams for your Rust projects.
# cargo-edit - A utility for adding (cargo-add), removing (cargo-rm), and upgrading (cargo-upgrade) cargo dependencies from the command line.
# cargo-expand - Print the result of macro expansion and #[derive] expansion.
# rustfmt - Format Rust code according to style guidelines.
# cargo-fuzz - Command-line wrapper for using libFuzzer
# cargo-generate - Create a new Rust project by leveraging a pre-existing git repository as a template.
# cargo-graph - Build GraphViz DOT files of dependency graphs. Unmaintained, consider using cargo-deps.
# cargo-info - Get crate information and details from crates.io
# cargo-license - List licensing info for the project's dependencies.
# cargo-lipo - Automatically create universal libraries for iOS.
# cargo-make - Rust task runner and build tool.
# cargo-modules - List a project's modules in a tree-like format.
# cargo-multi - Run a cargo command on multiple crates.
# cargo-open - Quickly open your crate in your editor.
# cargo-outdated - A cargo subcommand for displaying when Rust dependencies are out of date
# cargo-pkgbuild - Generate an Arch PKGBUILD for your crate.
# cargo-profiler - A cargo subcommand to profile your applications.
# cargo-release - Standardizes the release process of a cargo project.
# cargo-repro - Build and verify byte-for-byte reproducible Rust packages using a Cargo-based workflow (WIP).
# cargo-rpm - Build RPM releases of Rust projects using cargo.
# cargo-sandbox - Perform Cargo builds inside of a sandboxed environment (WIP).
# cargo-script - designed to let people quickly and easily run Rust "scripts" which can make use of Cargo's package ecosystem.
# cargo-tarpaulin - Code coverage tool for your Rust projects
# cargo-tomlfmt - Formatting Cargo.toml
# cargo-tree - List a project's dependencies in a tree-like format. Also supports an "inverted" mode to help determine why a specific crate is being pulled in.
# cargo-update - Check for cargo installed executables' newer versions and update as needed.
# cargo-urlcrate - Adds URLs of installing/downloading crates to Cargo output
# cargo-vendor - Vendors all crates.io dependencies into a local directory using Cargo's support for source replacement
# cargo-watch - Watch your repo for changes and build automatically.
# cargo-with - A cargo-subcommand making it easy to run the build artifacts produced by cargo run or cargo build through other tools such as gdb, strace, valgrind, rr, etc.
# cargo-x - A very simple third-party cargo subcommand to execute a custom command.
checkTravis: test

checkLocal: test

before:
	@echo "prepare start"
	cargo install --force cargo-audit
	cargo generate-lockfile
	@echo "prepare end"

test:
	@echo "test"
	cargo test -- --test-threads=8

audit:
	@echo "audit"
	cargo audit