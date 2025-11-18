# Auxin Version Control Extension - Main Module
# Copyright (c) 2024 Auxin Project

require 'json'
require 'open3'

module Auxin
  # Configuration
  CLI_TIMEOUT = 30 # seconds

  # State
  @dialog = nil
  @initialized = false

  class << self
    attr_accessor :dialog
  end

  # ==========================================================================
  # CLI Integration
  # ==========================================================================

  module CLI
    class << self
      # Execute an auxin CLI command
      # @param args [Array<String>] Command arguments
      # @return [Hash] Result with :success, :stdout, :stderr, :exit_code
      def execute(*args)
        command = ['auxin'] + args.flatten.map(&:to_s)

        begin
          stdout, stderr, status = Open3.capture3(*command)
          {
            success: status.success?,
            stdout: stdout.strip,
            stderr: stderr.strip,
            exit_code: status.exitstatus
          }
        rescue Errno::ENOENT
          {
            success: false,
            stdout: '',
            stderr: 'Auxin CLI not found. Please install auxin first.',
            exit_code: 127
          }
        rescue => e
          {
            success: false,
            stdout: '',
            stderr: "Error executing command: #{e.message}",
            exit_code: 1
          }
        end
      end

      # Initialize a repository for the current model
      def init(path)
        execute('init', '--type', 'sketchup', path)
      end

      # Check repository status
      def status(path)
        result = execute('status', '--path', path)
        return result unless result[:success]

        # Parse status output
        lines = result[:stdout].split("\n")
        status_data = {
          initialized: true,
          modified: [],
          staged: [],
          untracked: []
        }

        current_section = nil
        lines.each do |line|
          case line
          when /modified:/i
            current_section = :modified
          when /staged:/i
            current_section = :staged
          when /untracked:/i
            current_section = :untracked
          when /^\s+(.+)$/
            status_data[current_section] << $1.strip if current_section
          end
        end

        result[:data] = status_data
        result
      end

      # Create a commit with metadata
      def commit(path, message, metadata = {})
        args = ['commit', '-m', message, '--path', path]

        # Add SketchUp-specific metadata
        args += ['--units', metadata[:units]] if metadata[:units]
        args += ['--layers', metadata[:layers].to_s] if metadata[:layers]
        args += ['--components', metadata[:components].to_s] if metadata[:components]
        args += ['--groups', metadata[:groups].to_s] if metadata[:groups]
        args += ['--file-size', metadata[:file_size].to_s] if metadata[:file_size]
        args += ['--tags', metadata[:tags]] if metadata[:tags]

        execute(*args)
      end

      # Get commit history
      def log(path, limit = 50)
        result = execute('log', '--path', path, '--limit', limit.to_s, '--format', 'json')
        return result unless result[:success]

        begin
          result[:data] = JSON.parse(result[:stdout])
        rescue JSON::ParserError
          # Fallback to text parsing if JSON fails
          result[:data] = parse_log_text(result[:stdout])
        end

        result
      end

      # Restore to a specific commit
      def restore(path, commit_id)
        execute('restore', commit_id, '--path', path)
      end

      # Add files to staging
      def add(path, files = ['.'])
        args = ['add', '--path', path] + files.flatten
        execute(*args)
      end

      # Get diff between commits or working directory
      def diff(path, from_commit = nil, to_commit = nil)
        args = ['diff', '--path', path]
        args += ['--from', from_commit] if from_commit
        args += ['--to', to_commit] if to_commit
        execute(*args)
      end

      # Check if path is an initialized repository
      def is_repo?(path)
        result = execute('status', '--path', path)
        result[:success]
      end

      private

      def parse_log_text(text)
        commits = []
        current_commit = nil

        text.split("\n").each do |line|
          case line
          when /^commit\s+(\w+)/
            commits << current_commit if current_commit
            current_commit = { id: $1, message: '', metadata: {} }
          when /^Author:\s+(.+)/
            current_commit[:author] = $1 if current_commit
          when /^Date:\s+(.+)/
            current_commit[:date] = $1 if current_commit
          when /^(BPM|Sample Rate|Key|Units|Layers|Components|Groups):\s*(.+)/i
            current_commit[:metadata][$1.downcase.gsub(' ', '_').to_sym] = $2 if current_commit
          when /^\s{4}(.+)/
            current_commit[:message] += $1 + "\n" if current_commit
          end
        end

        commits << current_commit if current_commit
        commits
      end
    end
  end

  # ==========================================================================
  # Model Metadata Extraction
  # ==========================================================================

  module ModelMetadata
    class << self
      # Extract metadata from the current SketchUp model
      # @return [Hash] Model metadata
      def extract
        model = Sketchup.active_model
        return {} unless model

        {
          units: get_units(model),
          layers: count_layers(model),
          components: count_components(model),
          groups: count_groups(model),
          file_size: get_file_size(model),
          faces: count_faces(model),
          edges: count_edges(model),
          materials: count_materials(model)
        }
      end

      # Get a formatted summary of model metadata
      def summary
        data = extract
        return "No model loaded" if data.empty?

        [
          "Units: #{data[:units]}",
          "Layers/Tags: #{data[:layers]}",
          "Components: #{data[:components]}",
          "Groups: #{data[:groups]}",
          "Faces: #{data[:faces]}",
          "Edges: #{data[:edges]}",
          "Materials: #{data[:materials]}",
          "File Size: #{format_file_size(data[:file_size])}"
        ].join("\n")
      end

      private

      def get_units(model)
        options = model.options['UnitsOptions']
        case options['LengthUnit']
        when 0 then 'Inches'
        when 1 then 'Feet'
        when 2 then 'Millimeters'
        when 3 then 'Centimeters'
        when 4 then 'Meters'
        else 'Unknown'
        end
      end

      def count_layers(model)
        model.layers.count
      end

      def count_components(model)
        model.definitions.count { |d| !d.group? && d.instances.any? }
      end

      def count_groups(model)
        count = 0
        model.entities.grep(Sketchup::Group) { count += 1 }
        model.definitions.each do |defn|
          defn.entities.grep(Sketchup::Group) { count += 1 }
        end
        count
      end

      def count_faces(model)
        count = 0
        count_faces_recursive(model.entities, count)
      end

      def count_faces_recursive(entities, count = 0)
        entities.each do |entity|
          case entity
          when Sketchup::Face
            count += 1
          when Sketchup::Group, Sketchup::ComponentInstance
            defn = entity.respond_to?(:definition) ? entity.definition : entity.entities.parent
            count = count_faces_recursive(defn.entities, count) if defn
          end
        end
        count
      end

      def count_edges(model)
        count = 0
        model.entities.grep(Sketchup::Edge) { count += 1 }
        count
      end

      def count_materials(model)
        model.materials.count
      end

      def get_file_size(model)
        path = model.path
        return 0 if path.empty?
        File.exist?(path) ? File.size(path) : 0
      end

      def format_file_size(bytes)
        return "0 B" if bytes == 0
        units = ['B', 'KB', 'MB', 'GB']
        exp = (Math.log(bytes) / Math.log(1024)).to_i
        exp = units.length - 1 if exp >= units.length
        "%.1f %s" % [bytes.to_f / (1024 ** exp), units[exp]]
      end
    end
  end

  # ==========================================================================
  # Dialog Management
  # ==========================================================================

  module Dialog
    class << self
      def show
        if Auxin.dialog && Auxin.dialog.visible?
          Auxin.dialog.bring_to_front
          return
        end

        dialog_path = File.join(PLUGIN_DIR, 'dialogs', 'main_dialog.html')

        options = {
          dialog_title: 'Auxin Version Control',
          preferences_key: 'com.auxin.sketchup',
          scrollable: true,
          resizable: true,
          width: 450,
          height: 600,
          left: 100,
          top: 100,
          min_width: 350,
          min_height: 400,
          style: UI::HtmlDialog::STYLE_DIALOG
        }

        Auxin.dialog = UI::HtmlDialog.new(options)
        Auxin.dialog.set_file(dialog_path)

        # Register callbacks
        register_callbacks(Auxin.dialog)

        Auxin.dialog.show
      end

      def close
        Auxin.dialog&.close
        Auxin.dialog = nil
      end

      private

      def register_callbacks(dialog)
        # Get current model path
        dialog.add_action_callback('getModelPath') do |context|
          model = Sketchup.active_model
          path = model ? model.path : ''
          dialog.execute_script("onModelPath('#{escape_js(path)}')")
        end

        # Get model metadata
        dialog.add_action_callback('getMetadata') do |context|
          metadata = ModelMetadata.extract
          json = JSON.generate(metadata)
          dialog.execute_script("onMetadata(#{json})")
        end

        # Initialize repository
        dialog.add_action_callback('initRepo') do |context|
          model = Sketchup.active_model
          unless model && !model.path.empty?
            dialog.execute_script("onError('Please save your model first')")
            next
          end

          result = CLI.init(File.dirname(model.path))
          if result[:success]
            dialog.execute_script("onInitSuccess()")
          else
            dialog.execute_script("onError('#{escape_js(result[:stderr])}')")
          end
        end

        # Get repository status
        dialog.add_action_callback('getStatus') do |context|
          model = Sketchup.active_model
          unless model && !model.path.empty?
            dialog.execute_script("onStatus({initialized: false})")
            next
          end

          result = CLI.status(File.dirname(model.path))
          if result[:success]
            json = JSON.generate(result[:data] || { initialized: true })
            dialog.execute_script("onStatus(#{json})")
          else
            dialog.execute_script("onStatus({initialized: false})")
          end
        end

        # Create commit
        dialog.add_action_callback('commit') do |context, message, tags|
          model = Sketchup.active_model
          unless model && !model.path.empty?
            dialog.execute_script("onError('Please save your model first')")
            next
          end

          # Save model before commit
          model.save

          # Get metadata
          metadata = ModelMetadata.extract
          metadata[:tags] = tags if tags && !tags.empty?

          # Add files and commit
          repo_path = File.dirname(model.path)
          CLI.add(repo_path)
          result = CLI.commit(repo_path, message, metadata)

          if result[:success]
            dialog.execute_script("onCommitSuccess('#{escape_js(result[:stdout])}')")
          else
            dialog.execute_script("onError('#{escape_js(result[:stderr])}')")
          end
        end

        # Get commit history
        dialog.add_action_callback('getHistory') do |context, limit|
          model = Sketchup.active_model
          unless model && !model.path.empty?
            dialog.execute_script("onHistory([])")
            next
          end

          result = CLI.log(File.dirname(model.path), limit || 50)
          if result[:success]
            json = JSON.generate(result[:data] || [])
            dialog.execute_script("onHistory(#{json})")
          else
            dialog.execute_script("onHistory([])")
          end
        end

        # Restore to commit
        dialog.add_action_callback('restore') do |context, commit_id|
          model = Sketchup.active_model
          unless model && !model.path.empty?
            dialog.execute_script("onError('No model loaded')")
            next
          end

          # Confirm with user
          result = UI.messagebox(
            "Restore to commit #{commit_id[0..7]}?\n\nThis will discard any unsaved changes.",
            MB_YESNO
          )

          if result == IDYES
            restore_result = CLI.restore(File.dirname(model.path), commit_id)
            if restore_result[:success]
              # Reload the model
              Sketchup.open_file(model.path)
              dialog.execute_script("onRestoreSuccess()")
            else
              dialog.execute_script("onError('#{escape_js(restore_result[:stderr])}')")
            end
          end
        end

        # Show model info
        dialog.add_action_callback('showModelInfo') do |context|
          summary = ModelMetadata.summary
          UI.messagebox(summary, MB_OK, "Model Information")
        end

        # Open repository folder
        dialog.add_action_callback('openRepoFolder') do |context|
          model = Sketchup.active_model
          if model && !model.path.empty?
            folder = File.dirname(model.path)
            UI.openURL("file://#{folder}")
          end
        end

        # Check if auxin CLI is available
        dialog.add_action_callback('checkCLI') do |context|
          result = CLI.execute('--version')
          if result[:success]
            dialog.execute_script("onCLIAvailable('#{escape_js(result[:stdout])}')")
          else
            dialog.execute_script("onCLIUnavailable()")
          end
        end
      end

      def escape_js(str)
        str.to_s.gsub('\\', '\\\\').gsub("'", "\\\\'").gsub("\n", '\\n').gsub("\r", '')
      end
    end
  end

  # ==========================================================================
  # Menu and Toolbar
  # ==========================================================================

  module UI_Integration
    class << self
      def setup
        setup_menu
        setup_toolbar
        setup_observers
      end

      private

      def setup_menu
        menu = UI.menu('Extensions')
        auxin_menu = menu.add_submenu('Auxin')

        auxin_menu.add_item('Open Auxin Panel') { Dialog.show }
        auxin_menu.add_separator
        auxin_menu.add_item('Quick Commit...') { quick_commit }
        auxin_menu.add_item('View History') { Dialog.show }
        auxin_menu.add_separator
        auxin_menu.add_item('Initialize Repository') { init_repository }
        auxin_menu.add_item('Model Information') { show_model_info }
        auxin_menu.add_separator
        auxin_menu.add_item('About Auxin') { show_about }
      end

      def setup_toolbar
        toolbar = UI::Toolbar.new('Auxin')

        # Main panel button
        cmd_panel = UI::Command.new('Auxin Panel') { Dialog.show }
        cmd_panel.tooltip = 'Open Auxin Version Control Panel'
        cmd_panel.status_bar_text = 'Open the Auxin version control panel'
        cmd_panel.small_icon = File.join(PLUGIN_DIR, 'icons', 'auxin_small.png')
        cmd_panel.large_icon = File.join(PLUGIN_DIR, 'icons', 'auxin_large.png')
        toolbar.add_item(cmd_panel)

        # Quick commit button
        cmd_commit = UI::Command.new('Quick Commit') { quick_commit }
        cmd_commit.tooltip = 'Quick Commit Current Model'
        cmd_commit.status_bar_text = 'Create a quick commit with the current model state'
        cmd_commit.small_icon = File.join(PLUGIN_DIR, 'icons', 'commit_small.png')
        cmd_commit.large_icon = File.join(PLUGIN_DIR, 'icons', 'commit_large.png')
        toolbar.add_item(cmd_commit)

        toolbar.show
      end

      def setup_observers
        # Model observer for auto-status updates
        Sketchup.add_observer(AppObserver.new)
      end

      def quick_commit
        model = Sketchup.active_model
        unless model && !model.path.empty?
          UI.messagebox("Please save your model first.", MB_OK, "Auxin")
          return
        end

        # Check if repo is initialized
        unless CLI.is_repo?(File.dirname(model.path))
          result = UI.messagebox(
            "This model is not in an Auxin repository.\nWould you like to initialize one?",
            MB_YESNO,
            "Auxin"
          )
          if result == IDYES
            init_repository
          end
          return
        end

        # Prompt for commit message
        prompts = ['Commit Message:', 'Tags (optional):']
        defaults = ["Update #{File.basename(model.path)}", '']
        results = UI.inputbox(prompts, defaults, 'Quick Commit')

        return unless results

        message, tags = results
        return if message.empty?

        # Save and commit
        model.save
        metadata = ModelMetadata.extract
        metadata[:tags] = tags unless tags.empty?

        repo_path = File.dirname(model.path)
        CLI.add(repo_path)
        result = CLI.commit(repo_path, message, metadata)

        if result[:success]
          UI.messagebox("Commit created successfully!", MB_OK, "Auxin")
        else
          UI.messagebox("Commit failed: #{result[:stderr]}", MB_OK, "Auxin Error")
        end
      end

      def init_repository
        model = Sketchup.active_model
        unless model && !model.path.empty?
          UI.messagebox("Please save your model first.", MB_OK, "Auxin")
          return
        end

        repo_path = File.dirname(model.path)

        if CLI.is_repo?(repo_path)
          UI.messagebox("Repository already initialized.", MB_OK, "Auxin")
          return
        end

        result = CLI.init(repo_path)
        if result[:success]
          UI.messagebox("Repository initialized successfully!\n\n#{result[:stdout]}", MB_OK, "Auxin")
        else
          UI.messagebox("Failed to initialize repository:\n\n#{result[:stderr]}", MB_OK, "Auxin Error")
        end
      end

      def show_model_info
        summary = ModelMetadata.summary
        UI.messagebox(summary, MB_OK, "Model Information")
      end

      def show_about
        about_text = <<~ABOUT
          Auxin Version Control for SketchUp
          Version #{PLUGIN_VERSION}

          #{PLUGIN_DESCRIPTION}

          #{PLUGIN_COPYRIGHT}

          Powered by Oxen.ai
        ABOUT
        UI.messagebox(about_text, MB_OK, "About Auxin")
      end
    end
  end

  # ==========================================================================
  # Application Observer
  # ==========================================================================

  class AppObserver < Sketchup::AppObserver
    def onNewModel(model)
      refresh_dialog
    end

    def onOpenModel(model)
      refresh_dialog
    end

    private

    def refresh_dialog
      if Auxin.dialog && Auxin.dialog.visible?
        Auxin.dialog.execute_script("refreshAll()")
      end
    end
  end

  # ==========================================================================
  # Initialize Extension
  # ==========================================================================

  unless @initialized
    UI_Integration.setup
    @initialized = true
    puts "Auxin Version Control loaded successfully (v#{PLUGIN_VERSION})"
  end
end
