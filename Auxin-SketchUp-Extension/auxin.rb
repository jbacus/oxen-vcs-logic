# Auxin Version Control Extension for SketchUp
# Copyright (c) 2024 Auxin Project
# Licensed under MIT License

require 'sketchup.rb'
require 'extensions.rb'

module Auxin
  PLUGIN_ID = 'auxin'.freeze
  PLUGIN_NAME = 'Auxin Version Control'.freeze
  PLUGIN_VERSION = '1.0.0'.freeze
  PLUGIN_DESCRIPTION = 'Version control for SketchUp projects using Oxen.ai'.freeze
  PLUGIN_CREATOR = 'Auxin Project'.freeze
  PLUGIN_COPYRIGHT = "Copyright (c) 2024 #{PLUGIN_CREATOR}".freeze

  # Plugin paths
  PLUGIN_ROOT = File.dirname(__FILE__).freeze
  PLUGIN_DIR = File.join(PLUGIN_ROOT, 'auxin').freeze

  # Register the extension with SketchUp
  unless file_loaded?(__FILE__)
    extension = SketchupExtension.new(PLUGIN_NAME, File.join(PLUGIN_DIR, 'main.rb'))
    extension.description = PLUGIN_DESCRIPTION
    extension.version = PLUGIN_VERSION
    extension.creator = PLUGIN_CREATOR
    extension.copyright = PLUGIN_COPYRIGHT

    Sketchup.register_extension(extension, true)
    file_loaded(__FILE__)
  end
end
