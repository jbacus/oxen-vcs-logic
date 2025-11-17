# Homebrew Formula for OxVCS CLI
#
# Installation:
#   brew tap jbacus/oxenvcs
#   brew install oxenvcs-cli
#
# Or install directly from this formula:
#   brew install --build-from-source formula/oxenvcs-cli.rb

class OxenvcsLi < Formula
  desc "High-performance CLI wrapper for Oxen.ai version control of Logic Pro projects"
  homepage "https://github.com/jbacus/oxen-vcs-logic"
  url "https://github.com/jbacus/oxen-vcs-logic/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"  # Run: shasum -a 256 archive.tar.gz
  license "MIT"
  head "https://github.com/jbacus/oxen-vcs-logic.git", branch: "main"

  depends_on "rust" => :build

  # Optional runtime dependency
  # Users can also install with: pip3 install oxen-ai
  # depends_on "oxen" => :optional

  def install
    # Build from source
    cd "OxVCS-CLI-Wrapper" do
      system "cargo", "install", "--locked", "--root", prefix, "--path", "."
    end

    # Install shell completions
    cd "OxVCS-CLI-Wrapper" do
      # Generate completions
      system bin/"oxenvcs-cli", "completions", "bash"
      system bin/"oxenvcs-cli", "completions", "zsh"
      system bin/"oxenvcs-cli", "completions", "fish"

      # Install completions to appropriate directories
      bash_completion.install "completions/oxenvcs-cli.bash" => "oxenvcs-cli"
      zsh_completion.install "completions/_oxenvcs-cli"
      fish_completion.install "completions/oxenvcs-cli.fish"
    end

    # Install config template
    cd "OxVCS-CLI-Wrapper" do
      (prefix/"share/oxenvcs-cli").install "config.toml.example"
    end

    # Install documentation
    doc.install "README.md"
    doc.install "docs" if File.exist?("docs")
  end

  def caveats
    <<~EOS
      OxVCS CLI has been installed!

      To get started:
        1. Initialize a Logic Pro project:
           $ cd /path/to/your-project.logicx
           $ oxenvcs-cli init

        2. Create a config file (optional):
           $ mkdir -p ~/.oxenvcs
           $ cp #{prefix}/share/oxenvcs-cli/config.toml.example ~/.oxenvcs/config.toml

      Shell completions have been installed to:
        - Bash: #{bash_completion}/oxenvcs-cli
        - Zsh: #{zsh_completion}/_oxenvcs-cli
        - Fish: #{fish_completion}/oxenvcs-cli.fish

      IMPORTANT: OxVCS requires Oxen CLI to function.
      Install Oxen with:
        $ pip3 install oxen-ai
      OR:
        $ cargo install oxen

      For more information:
        $ oxenvcs-cli --help
        $ open https://github.com/jbacus/oxen-vcs-logic
    EOS
  end

  test do
    # Test binary runs
    assert_match "oxenvcs-cli", shell_output("#{bin}/oxenvcs-cli --version")

    # Test help command
    assert_match "High-performance CLI for Oxen.ai version control",
                 shell_output("#{bin}/oxenvcs-cli --help")

    # Test completions generation
    system bin/"oxenvcs-cli", "completions", "bash"

    # Test config template exists
    assert_predicate prefix/"share/oxenvcs-cli/config.toml.example", :exist?
  end
end
