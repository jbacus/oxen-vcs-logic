# Homebrew Formula for Auxin CLI
#
# Installation:
#   brew tap jbacus/auxin
#   brew install auxin
#
# Or install directly from this formula:
#   brew install --build-from-source formula/auxin.rb

class Auxin < Formula
  desc "Version control for Logic Pro projects, powered by Oxen.ai"
  homepage "https://github.com/jbacus/auxin"
  url "https://github.com/jbacus/auxin/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"  # Run: shasum -a 256 archive.tar.gz
  license "MIT"
  head "https://github.com/jbacus/auxin.git", branch: "main"

  depends_on "rust" => :build

  # Optional runtime dependency
  # Users can also install with: pip3 install oxen-ai
  # depends_on "oxen" => :optional

  def install
    # Build from source
    cd "Auxin-CLI-Wrapper" do
      system "cargo", "install", "--locked", "--root", prefix, "--path", "."
    end

    # Install shell completions
    cd "Auxin-CLI-Wrapper" do
      # Generate completions
      system bin/"auxin", "completions", "bash"
      system bin/"auxin", "completions", "zsh"
      system bin/"auxin", "completions", "fish"

      # Install completions to appropriate directories
      bash_completion.install "completions/auxin.bash" => "auxin"
      zsh_completion.install "completions/_auxin"
      fish_completion.install "completions/auxin.fish"
    end

    # Install config template
    cd "Auxin-CLI-Wrapper" do
      (prefix/"share/auxin").install "config.toml.example"
    end

    # Install documentation
    doc.install "README.md"
    doc.install "docs" if File.exist?("docs")
  end

  def caveats
    <<~EOS
      Auxin CLI has been installed!

      To get started:
        1. Initialize a Logic Pro project:
           $ cd /path/to/your-project.logicx
           $ auxin init

        2. Create a config file (optional):
           $ mkdir -p ~/.auxin
           $ cp #{prefix}/share/auxin/config.toml.example ~/.auxin/config.toml

      Shell completions have been installed to:
        - Bash: #{bash_completion}/auxin
        - Zsh: #{zsh_completion}/_auxin
        - Fish: #{fish_completion}/auxin.fish

      IMPORTANT: Auxin requires Oxen CLI to function.
      Install Oxen with:
        $ pip3 install oxen-ai
      OR:
        $ cargo install oxen

      For more information:
        $ auxin --help
        $ open https://github.com/jbacus/auxin
    EOS
  end

  test do
    # Test binary runs
    assert_match "auxin", shell_output("#{bin}/auxin --version")

    # Test help command
    assert_match "Version control for Logic Pro",
                 shell_output("#{bin}/auxin --help")

    # Test completions generation
    system bin/"auxin", "completions", "bash"

    # Test config template exists
    assert_predicate prefix/"share/auxin/config.toml.example", :exist?
  end
end
