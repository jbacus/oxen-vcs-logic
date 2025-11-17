
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'oxenvcs-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'oxenvcs-cli'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'oxenvcs-cli' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize a new Oxen repository for a Logic Pro project')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Stage changes to be committed')
            [CompletionResult]::new('commit', 'commit', [CompletionResultType]::ParameterValue, 'Create a commit with optional audio metadata')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Show commit history')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Restore project to a previous commit')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show repository status')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show detailed information about a commit')
            [CompletionResult]::new('diff', 'diff', [CompletionResultType]::ParameterValue, 'Show changes between commits or working directory')
            [CompletionResult]::new('compare', 'compare', [CompletionResultType]::ParameterValue, 'Compare metadata between two commits')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search commit history with advanced filtering')
            [CompletionResult]::new('lock', 'lock', [CompletionResultType]::ParameterValue, 'Manage project locks for team collaboration')
            [CompletionResult]::new('auth', 'auth', [CompletionResultType]::ParameterValue, 'Authenticate with Oxen Hub for remote collaboration')
            [CompletionResult]::new('metadata-diff', 'metadata-diff', [CompletionResultType]::ParameterValue, 'Compare metadata between two Logic Pro project versions')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Control the background daemon service')
            [CompletionResult]::new('hooks', 'hooks', [CompletionResultType]::ParameterValue, 'Manage workflow automation hooks')
            [CompletionResult]::new('console', 'console', [CompletionResultType]::ParameterValue, 'Launch interactive console for real-time monitoring')
            [CompletionResult]::new('activity', 'activity', [CompletionResultType]::ParameterValue, 'Show recent project activity timeline')
            [CompletionResult]::new('team', 'team', [CompletionResultType]::ParameterValue, 'Show team members and their contributions')
            [CompletionResult]::new('queue', 'queue', [CompletionResultType]::ParameterValue, 'Manage offline operation queue')
            [CompletionResult]::new('comment', 'comment', [CompletionResultType]::ParameterValue, 'Manage comments on commits')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;init' {
            [CompletionResult]::new('--logic', '--logic', [CompletionResultType]::ParameterName, 'Initialize for Logic Pro project (auto-detect and configure)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;add' {
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Stage all changes in the repository')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Stage all changes in the repository')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;commit' {
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'Commit message describing the changes')
            [CompletionResult]::new('--message', '--message', [CompletionResultType]::ParameterName, 'Commit message describing the changes')
            [CompletionResult]::new('--bpm', '--bpm', [CompletionResultType]::ParameterName, 'Beats per minute (tempo) of the project')
            [CompletionResult]::new('--sample-rate', '--sample-rate', [CompletionResultType]::ParameterName, 'Sample rate in Hz (e.g., 44100, 48000, 96000)')
            [CompletionResult]::new('--key', '--key', [CompletionResultType]::ParameterName, 'Key signature (e.g., ''C Major'', ''A Minor'', ''F# Minor'')')
            [CompletionResult]::new('--tags', '--tags', [CompletionResultType]::ParameterName, 'Tags for categorization (comma-separated, e.g., ''mixing,draft'')')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;log' {
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Maximum number of commits to display')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Maximum number of commits to display')
            [CompletionResult]::new('--bpm', '--bpm', [CompletionResultType]::ParameterName, 'Filter by BPM (e.g., 120, 128)')
            [CompletionResult]::new('--tag', '--tag', [CompletionResultType]::ParameterName, 'Filter by tag (e.g., ''mixing'', ''vocals'')')
            [CompletionResult]::new('--key', '--key', [CompletionResultType]::ParameterName, 'Filter by key signature (e.g., ''C Major'')')
            [CompletionResult]::new('--since', '--since', [CompletionResultType]::ParameterName, 'Show commits since date (YYYY-MM-DD)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;restore' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;show' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;diff' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;compare' {
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format (text, colored, json, compact)')
            [CompletionResult]::new('--plain', '--plain', [CompletionResultType]::ParameterName, 'Disable colored output')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;search' {
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format (list, compact, json)')
            [CompletionResult]::new('--ranked', '--ranked', [CompletionResultType]::ParameterName, 'Sort results by relevance score')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;lock' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('acquire', 'acquire', [CompletionResultType]::ParameterValue, 'Acquire exclusive lock for editing')
            [CompletionResult]::new('release', 'release', [CompletionResultType]::ParameterValue, 'Release the lock you currently hold')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current lock status')
            [CompletionResult]::new('break', 'break', [CompletionResultType]::ParameterValue, 'Force break an existing lock (admin only)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;lock;acquire' {
            [CompletionResult]::new('--timeout', '--timeout', [CompletionResultType]::ParameterName, 'Lock timeout in hours')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;lock;release' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;lock;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;lock;break' {
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Confirm you want to force break the lock')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;lock;help' {
            [CompletionResult]::new('acquire', 'acquire', [CompletionResultType]::ParameterValue, 'Acquire exclusive lock for editing')
            [CompletionResult]::new('release', 'release', [CompletionResultType]::ParameterValue, 'Release the lock you currently hold')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current lock status')
            [CompletionResult]::new('break', 'break', [CompletionResultType]::ParameterValue, 'Force break an existing lock (admin only)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;lock;help;acquire' {
            break
        }
        'oxenvcs-cli;lock;help;release' {
            break
        }
        'oxenvcs-cli;lock;help;status' {
            break
        }
        'oxenvcs-cli;lock;help;break' {
            break
        }
        'oxenvcs-cli;lock;help;help' {
            break
        }
        'oxenvcs-cli;auth' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Login to Oxen Hub with API credentials')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Logout from Oxen Hub')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication status')
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'Test authentication with Oxen Hub')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;auth;login' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;auth;logout' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;auth;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;auth;test' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;auth;help' {
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Login to Oxen Hub with API credentials')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Logout from Oxen Hub')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication status')
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'Test authentication with Oxen Hub')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;auth;help;login' {
            break
        }
        'oxenvcs-cli;auth;help;logout' {
            break
        }
        'oxenvcs-cli;auth;help;status' {
            break
        }
        'oxenvcs-cli;auth;help;test' {
            break
        }
        'oxenvcs-cli;auth;help;help' {
            break
        }
        'oxenvcs-cli;metadata-diff' {
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Output format (text or json)')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'Use colored output (default: auto-detect)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Include technical details in output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Include technical details in output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;daemon' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Check daemon status')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the daemon service')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop the daemon service')
            [CompletionResult]::new('restart', 'restart', [CompletionResultType]::ParameterValue, 'Restart the daemon service')
            [CompletionResult]::new('logs', 'logs', [CompletionResultType]::ParameterValue, 'Show daemon logs')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;daemon;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;daemon;start' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;daemon;stop' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;daemon;restart' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;daemon;logs' {
            [CompletionResult]::new('--lines', '--lines', [CompletionResultType]::ParameterName, 'Number of log lines to show')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;daemon;help' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Check daemon status')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the daemon service')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop the daemon service')
            [CompletionResult]::new('restart', 'restart', [CompletionResultType]::ParameterValue, 'Restart the daemon service')
            [CompletionResult]::new('logs', 'logs', [CompletionResultType]::ParameterValue, 'Show daemon logs')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;daemon;help;status' {
            break
        }
        'oxenvcs-cli;daemon;help;start' {
            break
        }
        'oxenvcs-cli;daemon;help;stop' {
            break
        }
        'oxenvcs-cli;daemon;help;restart' {
            break
        }
        'oxenvcs-cli;daemon;help;logs' {
            break
        }
        'oxenvcs-cli;daemon;help;help' {
            break
        }
        'oxenvcs-cli;hooks' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize hooks directory')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all installed hooks')
            [CompletionResult]::new('builtins', 'builtins', [CompletionResultType]::ParameterValue, 'List available built-in hooks')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install a built-in hook')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove an installed hook')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;hooks;init' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'oxenvcs-cli;hooks;list' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'oxenvcs-cli;hooks;builtins' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'oxenvcs-cli;hooks;install' {
            [CompletionResult]::new('--hook-type', '--hook-type', [CompletionResultType]::ParameterName, 'Hook type (pre-commit or post-commit)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'oxenvcs-cli;hooks;remove' {
            [CompletionResult]::new('--hook-type', '--hook-type', [CompletionResultType]::ParameterName, 'Hook type (pre-commit or post-commit)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'oxenvcs-cli;hooks;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize hooks directory')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all installed hooks')
            [CompletionResult]::new('builtins', 'builtins', [CompletionResultType]::ParameterValue, 'List available built-in hooks')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install a built-in hook')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove an installed hook')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;hooks;help;init' {
            break
        }
        'oxenvcs-cli;hooks;help;list' {
            break
        }
        'oxenvcs-cli;hooks;help;builtins' {
            break
        }
        'oxenvcs-cli;hooks;help;install' {
            break
        }
        'oxenvcs-cli;hooks;help;remove' {
            break
        }
        'oxenvcs-cli;hooks;help;help' {
            break
        }
        'oxenvcs-cli;console' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;activity' {
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Number of activities to show')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;team' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;queue' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show pending operations in the queue')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Manually sync all pending operations')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Clear completed operations from the queue')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a specific operation from the queue')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;queue;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;queue;sync' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;queue;clear' {
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Clear all operations including pending')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;queue;remove' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;queue;help' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show pending operations in the queue')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Manually sync all pending operations')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Clear completed operations from the queue')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a specific operation from the queue')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;queue;help;status' {
            break
        }
        'oxenvcs-cli;queue;help;sync' {
            break
        }
        'oxenvcs-cli;queue;help;clear' {
            break
        }
        'oxenvcs-cli;queue;help;remove' {
            break
        }
        'oxenvcs-cli;queue;help;help' {
            break
        }
        'oxenvcs-cli;comment' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a comment to a commit')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List comments on a commit')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;comment;add' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;comment;list' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;comment;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a comment to a commit')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List comments on a commit')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;comment;help;add' {
            break
        }
        'oxenvcs-cli;comment;help;list' {
            break
        }
        'oxenvcs-cli;comment;help;help' {
            break
        }
        'oxenvcs-cli;completions' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oxenvcs-cli;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize a new Oxen repository for a Logic Pro project')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Stage changes to be committed')
            [CompletionResult]::new('commit', 'commit', [CompletionResultType]::ParameterValue, 'Create a commit with optional audio metadata')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Show commit history')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Restore project to a previous commit')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show repository status')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show detailed information about a commit')
            [CompletionResult]::new('diff', 'diff', [CompletionResultType]::ParameterValue, 'Show changes between commits or working directory')
            [CompletionResult]::new('compare', 'compare', [CompletionResultType]::ParameterValue, 'Compare metadata between two commits')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search commit history with advanced filtering')
            [CompletionResult]::new('lock', 'lock', [CompletionResultType]::ParameterValue, 'Manage project locks for team collaboration')
            [CompletionResult]::new('auth', 'auth', [CompletionResultType]::ParameterValue, 'Authenticate with Oxen Hub for remote collaboration')
            [CompletionResult]::new('metadata-diff', 'metadata-diff', [CompletionResultType]::ParameterValue, 'Compare metadata between two Logic Pro project versions')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Control the background daemon service')
            [CompletionResult]::new('hooks', 'hooks', [CompletionResultType]::ParameterValue, 'Manage workflow automation hooks')
            [CompletionResult]::new('console', 'console', [CompletionResultType]::ParameterValue, 'Launch interactive console for real-time monitoring')
            [CompletionResult]::new('activity', 'activity', [CompletionResultType]::ParameterValue, 'Show recent project activity timeline')
            [CompletionResult]::new('team', 'team', [CompletionResultType]::ParameterValue, 'Show team members and their contributions')
            [CompletionResult]::new('queue', 'queue', [CompletionResultType]::ParameterValue, 'Manage offline operation queue')
            [CompletionResult]::new('comment', 'comment', [CompletionResultType]::ParameterValue, 'Manage comments on commits')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oxenvcs-cli;help;init' {
            break
        }
        'oxenvcs-cli;help;add' {
            break
        }
        'oxenvcs-cli;help;commit' {
            break
        }
        'oxenvcs-cli;help;log' {
            break
        }
        'oxenvcs-cli;help;restore' {
            break
        }
        'oxenvcs-cli;help;status' {
            break
        }
        'oxenvcs-cli;help;show' {
            break
        }
        'oxenvcs-cli;help;diff' {
            break
        }
        'oxenvcs-cli;help;compare' {
            break
        }
        'oxenvcs-cli;help;search' {
            break
        }
        'oxenvcs-cli;help;lock' {
            [CompletionResult]::new('acquire', 'acquire', [CompletionResultType]::ParameterValue, 'Acquire exclusive lock for editing')
            [CompletionResult]::new('release', 'release', [CompletionResultType]::ParameterValue, 'Release the lock you currently hold')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current lock status')
            [CompletionResult]::new('break', 'break', [CompletionResultType]::ParameterValue, 'Force break an existing lock (admin only)')
            break
        }
        'oxenvcs-cli;help;lock;acquire' {
            break
        }
        'oxenvcs-cli;help;lock;release' {
            break
        }
        'oxenvcs-cli;help;lock;status' {
            break
        }
        'oxenvcs-cli;help;lock;break' {
            break
        }
        'oxenvcs-cli;help;auth' {
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Login to Oxen Hub with API credentials')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Logout from Oxen Hub')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication status')
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'Test authentication with Oxen Hub')
            break
        }
        'oxenvcs-cli;help;auth;login' {
            break
        }
        'oxenvcs-cli;help;auth;logout' {
            break
        }
        'oxenvcs-cli;help;auth;status' {
            break
        }
        'oxenvcs-cli;help;auth;test' {
            break
        }
        'oxenvcs-cli;help;metadata-diff' {
            break
        }
        'oxenvcs-cli;help;daemon' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Check daemon status')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the daemon service')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop the daemon service')
            [CompletionResult]::new('restart', 'restart', [CompletionResultType]::ParameterValue, 'Restart the daemon service')
            [CompletionResult]::new('logs', 'logs', [CompletionResultType]::ParameterValue, 'Show daemon logs')
            break
        }
        'oxenvcs-cli;help;daemon;status' {
            break
        }
        'oxenvcs-cli;help;daemon;start' {
            break
        }
        'oxenvcs-cli;help;daemon;stop' {
            break
        }
        'oxenvcs-cli;help;daemon;restart' {
            break
        }
        'oxenvcs-cli;help;daemon;logs' {
            break
        }
        'oxenvcs-cli;help;hooks' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize hooks directory')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all installed hooks')
            [CompletionResult]::new('builtins', 'builtins', [CompletionResultType]::ParameterValue, 'List available built-in hooks')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install a built-in hook')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove an installed hook')
            break
        }
        'oxenvcs-cli;help;hooks;init' {
            break
        }
        'oxenvcs-cli;help;hooks;list' {
            break
        }
        'oxenvcs-cli;help;hooks;builtins' {
            break
        }
        'oxenvcs-cli;help;hooks;install' {
            break
        }
        'oxenvcs-cli;help;hooks;remove' {
            break
        }
        'oxenvcs-cli;help;console' {
            break
        }
        'oxenvcs-cli;help;activity' {
            break
        }
        'oxenvcs-cli;help;team' {
            break
        }
        'oxenvcs-cli;help;queue' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show pending operations in the queue')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Manually sync all pending operations')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Clear completed operations from the queue')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a specific operation from the queue')
            break
        }
        'oxenvcs-cli;help;queue;status' {
            break
        }
        'oxenvcs-cli;help;queue;sync' {
            break
        }
        'oxenvcs-cli;help;queue;clear' {
            break
        }
        'oxenvcs-cli;help;queue;remove' {
            break
        }
        'oxenvcs-cli;help;comment' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a comment to a commit')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List comments on a commit')
            break
        }
        'oxenvcs-cli;help;comment;add' {
            break
        }
        'oxenvcs-cli;help;comment;list' {
            break
        }
        'oxenvcs-cli;help;completions' {
            break
        }
        'oxenvcs-cli;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
