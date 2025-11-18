# Auxin Version Control for Blender
# Copyright (c) 2025 Auxin Project
# SPDX-License-Identifier: MIT

bl_info = {
    "name": "Auxin Version Control",
    "author": "Auxin Project",
    "version": (1, 0, 0),
    "blender": (3, 0, 0),
    "location": "View3D > Sidebar > Auxin",
    "description": "Version control for Blender projects using Auxin/Oxen",
    "warning": "",
    "doc_url": "https://github.com/jbacus/auxin",
    "category": "System",
}

import bpy
import subprocess
import json
import os
import threading
from bpy.props import (
    StringProperty,
    BoolProperty,
    IntProperty,
    EnumProperty,
    CollectionProperty,
)
from bpy.types import (
    Operator,
    Panel,
    PropertyGroup,
    AddonPreferences,
)

# =============================================================================
# Utility Functions
# =============================================================================

def get_project_path():
    """Get the directory containing the current .blend file."""
    filepath = bpy.data.filepath
    if filepath:
        return os.path.dirname(filepath)
    return None

def get_blend_file():
    """Get the current .blend file path."""
    return bpy.data.filepath

def run_auxin_command(args, cwd=None):
    """Run an auxin CLI command and return the result."""
    if cwd is None:
        cwd = get_project_path()

    if not cwd:
        return {"success": False, "error": "No project directory - save your file first"}

    try:
        cmd = ["auxin"] + args
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=60
        )

        if result.returncode == 0:
            return {"success": True, "output": result.stdout, "stderr": result.stderr}
        else:
            return {"success": False, "error": result.stderr or result.stdout}
    except FileNotFoundError:
        return {"success": False, "error": "auxin CLI not found. Please install auxin."}
    except subprocess.TimeoutExpired:
        return {"success": False, "error": "Command timed out"}
    except Exception as e:
        return {"success": False, "error": str(e)}

def extract_scene_metadata():
    """Extract metadata from the current Blender scene for commit messages."""
    scene = bpy.context.scene

    # Count objects by type
    mesh_count = len([obj for obj in bpy.data.objects if obj.type == 'MESH'])
    light_count = len([obj for obj in bpy.data.objects if obj.type == 'LIGHT'])
    camera_count = len([obj for obj in bpy.data.objects if obj.type == 'CAMERA'])
    total_objects = len(bpy.data.objects)

    # Get other stats
    material_count = len(bpy.data.materials)
    scene_count = len(bpy.data.scenes)

    # Render settings
    render = scene.render
    render_engine = render.engine
    resolution = (render.resolution_x, render.resolution_y)

    # Animation settings
    frame_start = scene.frame_start
    frame_end = scene.frame_end
    fps = scene.render.fps

    # Blender version
    blender_version = f"{bpy.app.version[0]}.{bpy.app.version[1]}.{bpy.app.version[2]}"

    # File size
    filepath = bpy.data.filepath
    file_size = 0
    if filepath and os.path.exists(filepath):
        file_size = os.path.getsize(filepath)

    return {
        "scene_count": scene_count,
        "active_scene": scene.name,
        "mesh_count": mesh_count,
        "light_count": light_count,
        "camera_count": camera_count,
        "object_count": total_objects,
        "material_count": material_count,
        "render_engine": render_engine,
        "resolution_x": resolution[0],
        "resolution_y": resolution[1],
        "frame_start": frame_start,
        "frame_end": frame_end,
        "fps": fps,
        "blender_version": blender_version,
        "file_size": file_size,
    }

def format_file_size(size_bytes):
    """Format file size in human-readable format."""
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024 * 1024:
        return f"{size_bytes / 1024:.1f} KB"
    elif size_bytes < 1024 * 1024 * 1024:
        return f"{size_bytes / (1024 * 1024):.1f} MB"
    else:
        return f"{size_bytes / (1024 * 1024 * 1024):.2f} GB"

# =============================================================================
# Property Groups
# =============================================================================

class AuxinCommitEntry(PropertyGroup):
    """Property group for a single commit entry."""
    commit_id: StringProperty(name="Commit ID")
    message: StringProperty(name="Message")
    author: StringProperty(name="Author")
    date: StringProperty(name="Date")
    is_selected: BoolProperty(name="Selected", default=False)

class AuxinSceneProperties(PropertyGroup):
    """Scene-level properties for Auxin."""
    commit_message: StringProperty(
        name="Commit Message",
        description="Message describing your changes",
        default=""
    )

    include_metadata: BoolProperty(
        name="Include Metadata",
        description="Include scene metadata in commit message",
        default=True
    )

    auto_stage: BoolProperty(
        name="Auto Stage",
        description="Automatically stage all changes before commit",
        default=True
    )

    repo_initialized: BoolProperty(
        name="Repository Initialized",
        description="Whether the repository has been initialized",
        default=False
    )

    last_status: StringProperty(
        name="Last Status",
        description="Last status message from auxin",
        default=""
    )

    is_locked: BoolProperty(
        name="Is Locked",
        description="Whether the project is currently locked",
        default=False
    )

    lock_holder: StringProperty(
        name="Lock Holder",
        description="Who currently holds the lock",
        default=""
    )

    commits: CollectionProperty(type=AuxinCommitEntry)

    active_commit_index: IntProperty(name="Active Commit Index", default=0)

    tag_input: StringProperty(
        name="Tags",
        description="Comma-separated tags for the commit",
        default=""
    )

# =============================================================================
# Operators
# =============================================================================

class AUXIN_OT_init_repository(Operator):
    """Initialize a new Auxin repository for the current Blender project"""
    bl_idname = "auxin.init_repository"
    bl_label = "Initialize Repository"
    bl_description = "Initialize Auxin version control for this project"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        filepath = get_blend_file()
        if not filepath:
            self.report({'ERROR'}, "Save your file first")
            return {'CANCELLED'}

        result = run_auxin_command(["init", "--type", "blender", filepath])

        if result["success"]:
            context.scene.auxin.repo_initialized = True
            context.scene.auxin.last_status = "Repository initialized"
            self.report({'INFO'}, "Repository initialized successfully")
        else:
            self.report({'ERROR'}, result["error"])
            return {'CANCELLED'}

        return {'FINISHED'}

class AUXIN_OT_add_all(Operator):
    """Stage all changes for the next commit"""
    bl_idname = "auxin.add_all"
    bl_label = "Stage All Changes"
    bl_description = "Stage all changes for the next commit"
    bl_options = {'REGISTER'}

    def execute(self, context):
        # First save the file
        if bpy.data.is_dirty:
            bpy.ops.wm.save_mainfile()

        result = run_auxin_command(["add", "--all"])

        if result["success"]:
            context.scene.auxin.last_status = "All changes staged"
            self.report({'INFO'}, "All changes staged")
        else:
            self.report({'ERROR'}, result["error"])
            return {'CANCELLED'}

        return {'FINISHED'}

class AUXIN_OT_commit(Operator):
    """Create a commit with the current changes"""
    bl_idname = "auxin.commit"
    bl_label = "Commit Changes"
    bl_description = "Create a commit with the current changes"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        props = context.scene.auxin

        if not props.commit_message.strip():
            self.report({'ERROR'}, "Please enter a commit message")
            return {'CANCELLED'}

        # Save the file first
        if bpy.data.is_dirty:
            bpy.ops.wm.save_mainfile()

        # Auto stage if enabled
        if props.auto_stage:
            stage_result = run_auxin_command(["add", "--all"])
            if not stage_result["success"]:
                self.report({'WARNING'}, f"Staging warning: {stage_result['error']}")

        # Build commit command with metadata
        message = props.commit_message.strip()

        if props.include_metadata:
            metadata = extract_scene_metadata()

            # Format metadata as part of the message
            metadata_lines = [
                f"\n\nScenes: {metadata['scene_count']}",
                f"Active Scene: {metadata['active_scene']}",
                f"Objects: {metadata['object_count']}",
                f"Meshes: {metadata['mesh_count']}",
                f"Lights: {metadata['light_count']}",
                f"Cameras: {metadata['camera_count']}",
                f"Materials: {metadata['material_count']}",
                f"Render Engine: {metadata['render_engine']}",
                f"Resolution: {metadata['resolution_x']}x{metadata['resolution_y']}",
                f"Frame Range: {metadata['frame_start']}-{metadata['frame_end']}",
                f"FPS: {metadata['fps']}",
                f"Blender Version: {metadata['blender_version']}",
            ]

            if metadata['file_size'] > 0:
                size_mb = metadata['file_size'] / (1024 * 1024)
                metadata_lines.append(f"File Size: {size_mb:.2f} MB")

            if props.tag_input.strip():
                metadata_lines.append(f"Tags: {props.tag_input.strip()}")

            message += "".join(metadata_lines)

        result = run_auxin_command(["commit", "-m", message])

        if result["success"]:
            props.commit_message = ""
            props.tag_input = ""
            props.last_status = "Commit created successfully"
            self.report({'INFO'}, "Commit created successfully")

            # Refresh commit history
            bpy.ops.auxin.refresh_history()
        else:
            self.report({'ERROR'}, result["error"])
            return {'CANCELLED'}

        return {'FINISHED'}

class AUXIN_OT_refresh_history(Operator):
    """Refresh the commit history"""
    bl_idname = "auxin.refresh_history"
    bl_label = "Refresh History"
    bl_description = "Refresh the commit history from the repository"
    bl_options = {'REGISTER'}

    def execute(self, context):
        props = context.scene.auxin

        result = run_auxin_command(["log", "--json"])

        if result["success"]:
            # Clear existing commits
            props.commits.clear()

            # Parse JSON output
            try:
                output = result["output"].strip()
                if output:
                    # Try to parse as JSON array
                    commits_data = json.loads(output)

                    for commit_data in commits_data:
                        entry = props.commits.add()
                        entry.commit_id = commit_data.get("id", "")[:8]
                        entry.message = commit_data.get("message", "").split("\n")[0][:50]
                        entry.author = commit_data.get("author", "")
                        entry.date = commit_data.get("date", "")
            except json.JSONDecodeError:
                # Fallback: parse plain text output
                lines = result["output"].strip().split("\n")
                for i, line in enumerate(lines[:20]):  # Limit to 20 entries
                    if line.strip():
                        entry = props.commits.add()
                        parts = line.split(" ", 1)
                        entry.commit_id = parts[0][:8] if parts else ""
                        entry.message = parts[1][:50] if len(parts) > 1 else ""

            props.last_status = f"Loaded {len(props.commits)} commits"
            self.report({'INFO'}, f"Loaded {len(props.commits)} commits")
        else:
            self.report({'WARNING'}, f"Could not load history: {result['error']}")

        return {'FINISHED'}

class AUXIN_OT_restore_commit(Operator):
    """Restore the project to a previous commit"""
    bl_idname = "auxin.restore_commit"
    bl_label = "Restore Commit"
    bl_description = "Restore the project to the selected commit"
    bl_options = {'REGISTER', 'UNDO'}

    commit_id: StringProperty(name="Commit ID")

    def invoke(self, context, event):
        props = context.scene.auxin
        if 0 <= props.active_commit_index < len(props.commits):
            self.commit_id = props.commits[props.active_commit_index].commit_id
        return context.window_manager.invoke_confirm(self, event)

    def execute(self, context):
        if not self.commit_id:
            self.report({'ERROR'}, "No commit selected")
            return {'CANCELLED'}

        # Check for unsaved changes
        if bpy.data.is_dirty:
            self.report({'WARNING'}, "Saving current changes before restore")
            bpy.ops.wm.save_mainfile()

        result = run_auxin_command(["restore", self.commit_id])

        if result["success"]:
            context.scene.auxin.last_status = f"Restored to {self.commit_id}"
            self.report({'INFO'}, f"Restored to commit {self.commit_id}")

            # Reload the file
            bpy.ops.wm.revert_mainfile()
        else:
            self.report({'ERROR'}, result["error"])
            return {'CANCELLED'}

        return {'FINISHED'}

class AUXIN_OT_check_status(Operator):
    """Check repository status"""
    bl_idname = "auxin.check_status"
    bl_label = "Check Status"
    bl_description = "Check the current repository status"
    bl_options = {'REGISTER'}

    def execute(self, context):
        props = context.scene.auxin

        result = run_auxin_command(["status"])

        if result["success"]:
            # Parse status output to check if initialized
            output = result["output"]
            if "not an oxen repository" in output.lower():
                props.repo_initialized = False
                props.last_status = "Not initialized"
            else:
                props.repo_initialized = True
                # Count changes
                lines = output.strip().split("\n")
                props.last_status = f"{len(lines)} file(s) changed"

            self.report({'INFO'}, props.last_status)
        else:
            if "not an oxen repository" in result["error"].lower():
                props.repo_initialized = False
                props.last_status = "Not initialized"
            else:
                props.last_status = result["error"]
            self.report({'INFO'}, props.last_status)

        return {'FINISHED'}

class AUXIN_OT_acquire_lock(Operator):
    """Acquire exclusive lock for editing"""
    bl_idname = "auxin.acquire_lock"
    bl_label = "Acquire Lock"
    bl_description = "Acquire exclusive lock to edit this project"
    bl_options = {'REGISTER'}

    timeout: IntProperty(
        name="Timeout (hours)",
        description="Lock timeout in hours",
        default=4,
        min=1,
        max=24
    )

    def execute(self, context):
        props = context.scene.auxin

        result = run_auxin_command(["lock", "acquire", "--timeout", str(self.timeout)])

        if result["success"]:
            props.is_locked = True
            props.lock_holder = "You"
            props.last_status = "Lock acquired"
            self.report({'INFO'}, "Lock acquired successfully")
        else:
            self.report({'ERROR'}, result["error"])
            return {'CANCELLED'}

        return {'FINISHED'}

class AUXIN_OT_release_lock(Operator):
    """Release the current lock"""
    bl_idname = "auxin.release_lock"
    bl_label = "Release Lock"
    bl_description = "Release the lock you currently hold"
    bl_options = {'REGISTER'}

    def execute(self, context):
        props = context.scene.auxin

        result = run_auxin_command(["lock", "release"])

        if result["success"]:
            props.is_locked = False
            props.lock_holder = ""
            props.last_status = "Lock released"
            self.report({'INFO'}, "Lock released successfully")
        else:
            self.report({'ERROR'}, result["error"])
            return {'CANCELLED'}

        return {'FINISHED'}

class AUXIN_OT_check_lock_status(Operator):
    """Check lock status"""
    bl_idname = "auxin.check_lock_status"
    bl_label = "Check Lock Status"
    bl_description = "Check the current lock status"
    bl_options = {'REGISTER'}

    def execute(self, context):
        props = context.scene.auxin

        result = run_auxin_command(["lock", "status"])

        if result["success"]:
            output = result["output"].lower()
            if "not locked" in output or "no lock" in output:
                props.is_locked = False
                props.lock_holder = ""
                props.last_status = "No lock held"
            else:
                props.is_locked = True
                # Try to parse lock holder from output
                props.last_status = "Project is locked"

            self.report({'INFO'}, props.last_status)
        else:
            self.report({'WARNING'}, result["error"])

        return {'FINISHED'}

class AUXIN_OT_push(Operator):
    """Push commits to remote repository"""
    bl_idname = "auxin.push"
    bl_label = "Push"
    bl_description = "Push local commits to the remote repository"
    bl_options = {'REGISTER'}

    def execute(self, context):
        props = context.scene.auxin

        result = run_auxin_command(["push"])

        if result["success"]:
            props.last_status = "Pushed to remote"
            self.report({'INFO'}, "Pushed successfully")
        else:
            self.report({'ERROR'}, result["error"])
            return {'CANCELLED'}

        return {'FINISHED'}

class AUXIN_OT_pull(Operator):
    """Pull commits from remote repository"""
    bl_idname = "auxin.pull"
    bl_label = "Pull"
    bl_description = "Pull commits from the remote repository"
    bl_options = {'REGISTER'}

    def execute(self, context):
        props = context.scene.auxin

        result = run_auxin_command(["pull"])

        if result["success"]:
            props.last_status = "Pulled from remote"
            self.report({'INFO'}, "Pulled successfully")

            # Reload the file if it was updated
            bpy.ops.wm.revert_mainfile()
        else:
            self.report({'ERROR'}, result["error"])
            return {'CANCELLED'}

        return {'FINISHED'}

class AUXIN_OT_open_in_terminal(Operator):
    """Open terminal in project directory"""
    bl_idname = "auxin.open_terminal"
    bl_label = "Open Terminal"
    bl_description = "Open a terminal in the project directory"
    bl_options = {'REGISTER'}

    def execute(self, context):
        project_path = get_project_path()
        if not project_path:
            self.report({'ERROR'}, "Save your file first")
            return {'CANCELLED'}

        import platform
        system = platform.system()

        try:
            if system == "Darwin":  # macOS
                subprocess.Popen(["open", "-a", "Terminal", project_path])
            elif system == "Linux":
                # Try common terminal emulators
                terminals = ["gnome-terminal", "konsole", "xterm", "terminator"]
                for term in terminals:
                    try:
                        subprocess.Popen([term, "--working-directory", project_path])
                        break
                    except FileNotFoundError:
                        continue
            elif system == "Windows":
                subprocess.Popen(["cmd", "/c", "start", "cmd", "/k", f"cd /d {project_path}"])

            self.report({'INFO'}, f"Opened terminal in {project_path}")
        except Exception as e:
            self.report({'ERROR'}, str(e))
            return {'CANCELLED'}

        return {'FINISHED'}

# =============================================================================
# UI Panels
# =============================================================================

class AUXIN_PT_main_panel(Panel):
    """Main Auxin panel in the 3D View sidebar"""
    bl_label = "Auxin Version Control"
    bl_idname = "AUXIN_PT_main_panel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Auxin'

    def draw(self, context):
        layout = self.layout
        props = context.scene.auxin

        # File status
        filepath = get_blend_file()
        if filepath:
            filename = os.path.basename(filepath)
            layout.label(text=f"File: {filename}", icon='FILE_BLEND')
        else:
            layout.label(text="File: Not saved", icon='ERROR')
            layout.operator("wm.save_as_mainfile", text="Save File", icon='FILE_TICK')
            return

        # Repository status
        box = layout.box()
        row = box.row()
        if props.repo_initialized:
            row.label(text="Repository: Initialized", icon='CHECKMARK')
        else:
            row.label(text="Repository: Not initialized", icon='ERROR')
            box.operator("auxin.init_repository", text="Initialize", icon='ADD')
            return

        # Status
        if props.last_status:
            box.label(text=props.last_status, icon='INFO')

        # Quick actions
        row = box.row(align=True)
        row.operator("auxin.check_status", text="", icon='FILE_REFRESH')
        row.operator("auxin.add_all", text="Stage All", icon='CHECKMARK')

class AUXIN_PT_commit_panel(Panel):
    """Commit panel"""
    bl_label = "Commit"
    bl_idname = "AUXIN_PT_commit_panel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Auxin'
    bl_parent_id = "AUXIN_PT_main_panel"
    bl_options = {'DEFAULT_CLOSED'}

    @classmethod
    def poll(cls, context):
        return context.scene.auxin.repo_initialized

    def draw(self, context):
        layout = self.layout
        props = context.scene.auxin

        # Commit message
        layout.label(text="Commit Message:")
        layout.prop(props, "commit_message", text="")

        # Tags
        layout.prop(props, "tag_input", text="Tags")

        # Options
        col = layout.column(align=True)
        col.prop(props, "include_metadata")
        col.prop(props, "auto_stage")

        # Scene info
        if props.include_metadata:
            box = layout.box()
            box.label(text="Scene Metadata:", icon='INFO')

            metadata = extract_scene_metadata()
            col = box.column(align=True)
            col.scale_y = 0.8
            col.label(text=f"Objects: {metadata['object_count']}")
            col.label(text=f"Materials: {metadata['material_count']}")
            col.label(text=f"Engine: {metadata['render_engine']}")
            col.label(text=f"Frames: {metadata['frame_start']}-{metadata['frame_end']}")
            if metadata['file_size'] > 0:
                col.label(text=f"Size: {format_file_size(metadata['file_size'])}")

        # Commit button
        row = layout.row()
        row.scale_y = 1.5
        row.operator("auxin.commit", text="Commit", icon='EXPORT')

class AUXIN_PT_history_panel(Panel):
    """Commit history panel"""
    bl_label = "History"
    bl_idname = "AUXIN_PT_history_panel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Auxin'
    bl_parent_id = "AUXIN_PT_main_panel"
    bl_options = {'DEFAULT_CLOSED'}

    @classmethod
    def poll(cls, context):
        return context.scene.auxin.repo_initialized

    def draw(self, context):
        layout = self.layout
        props = context.scene.auxin

        # Refresh button
        row = layout.row()
        row.operator("auxin.refresh_history", text="Refresh", icon='FILE_REFRESH')

        # Commit list
        if props.commits:
            box = layout.box()
            for i, commit in enumerate(props.commits):
                row = box.row()

                # Highlight selected
                if i == props.active_commit_index:
                    row.alert = True

                col = row.column()
                col.scale_y = 0.8
                col.label(text=f"{commit.commit_id}: {commit.message}")

                # Select button
                op = row.operator("auxin.select_commit", text="", icon='RESTRICT_SELECT_OFF')
                op.index = i

            # Restore button
            if len(props.commits) > 0:
                layout.separator()
                layout.operator("auxin.restore_commit", text="Restore Selected", icon='LOOP_BACK')
        else:
            layout.label(text="No commits yet", icon='INFO')

class AUXIN_OT_select_commit(Operator):
    """Select a commit from the history"""
    bl_idname = "auxin.select_commit"
    bl_label = "Select Commit"
    bl_options = {'REGISTER', 'INTERNAL'}

    index: IntProperty()

    def execute(self, context):
        context.scene.auxin.active_commit_index = self.index
        return {'FINISHED'}

class AUXIN_PT_remote_panel(Panel):
    """Remote operations panel"""
    bl_label = "Remote"
    bl_idname = "AUXIN_PT_remote_panel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Auxin'
    bl_parent_id = "AUXIN_PT_main_panel"
    bl_options = {'DEFAULT_CLOSED'}

    @classmethod
    def poll(cls, context):
        return context.scene.auxin.repo_initialized

    def draw(self, context):
        layout = self.layout

        row = layout.row(align=True)
        row.operator("auxin.push", text="Push", icon='EXPORT')
        row.operator("auxin.pull", text="Pull", icon='IMPORT')

class AUXIN_PT_lock_panel(Panel):
    """Lock management panel"""
    bl_label = "Lock Management"
    bl_idname = "AUXIN_PT_lock_panel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Auxin'
    bl_parent_id = "AUXIN_PT_main_panel"
    bl_options = {'DEFAULT_CLOSED'}

    @classmethod
    def poll(cls, context):
        return context.scene.auxin.repo_initialized

    def draw(self, context):
        layout = self.layout
        props = context.scene.auxin

        # Lock status
        box = layout.box()
        if props.is_locked:
            box.label(text=f"Locked by: {props.lock_holder}", icon='LOCKED')
            box.operator("auxin.release_lock", text="Release Lock", icon='UNLOCKED')
        else:
            box.label(text="Not locked", icon='UNLOCKED')
            box.operator("auxin.acquire_lock", text="Acquire Lock", icon='LOCKED')

        # Check status
        layout.operator("auxin.check_lock_status", text="Check Status", icon='FILE_REFRESH')

class AUXIN_PT_tools_panel(Panel):
    """Additional tools panel"""
    bl_label = "Tools"
    bl_idname = "AUXIN_PT_tools_panel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Auxin'
    bl_parent_id = "AUXIN_PT_main_panel"
    bl_options = {'DEFAULT_CLOSED'}

    def draw(self, context):
        layout = self.layout
        layout.operator("auxin.open_terminal", text="Open Terminal", icon='CONSOLE')

# =============================================================================
# Addon Preferences
# =============================================================================

class AuxinAddonPreferences(AddonPreferences):
    bl_idname = __name__

    auxin_path: StringProperty(
        name="Auxin CLI Path",
        description="Path to the auxin CLI executable (leave empty for system PATH)",
        default="",
        subtype='FILE_PATH'
    )

    auto_check_status: BoolProperty(
        name="Auto Check Status",
        description="Automatically check repository status when opening files",
        default=True
    )

    def draw(self, context):
        layout = self.layout

        layout.prop(self, "auxin_path")
        layout.prop(self, "auto_check_status")

        layout.separator()
        layout.label(text="For more information, visit the Auxin documentation:")
        layout.operator("wm.url_open", text="Auxin Documentation").url = "https://github.com/jbacus/auxin"

# =============================================================================
# Auto-save Handler
# =============================================================================

@bpy.app.handlers.persistent
def auxin_save_handler(dummy):
    """Handler called after saving a file."""
    # Check if auto-staging is enabled in preferences
    try:
        prefs = bpy.context.preferences.addons[__name__].preferences
        if not prefs.auto_check_status:
            return
    except:
        pass

    # Update status after save
    if bpy.context.scene.auxin.repo_initialized:
        bpy.ops.auxin.check_status()

@bpy.app.handlers.persistent
def auxin_load_handler(dummy):
    """Handler called after loading a file."""
    # Check repository status when file is loaded
    try:
        prefs = bpy.context.preferences.addons[__name__].preferences
        if not prefs.auto_check_status:
            return
    except:
        pass

    # Check if this is an auxin repository
    if get_blend_file():
        bpy.ops.auxin.check_status()

# =============================================================================
# Registration
# =============================================================================

classes = (
    AuxinCommitEntry,
    AuxinSceneProperties,
    AUXIN_OT_init_repository,
    AUXIN_OT_add_all,
    AUXIN_OT_commit,
    AUXIN_OT_refresh_history,
    AUXIN_OT_restore_commit,
    AUXIN_OT_check_status,
    AUXIN_OT_acquire_lock,
    AUXIN_OT_release_lock,
    AUXIN_OT_check_lock_status,
    AUXIN_OT_push,
    AUXIN_OT_pull,
    AUXIN_OT_open_in_terminal,
    AUXIN_OT_select_commit,
    AUXIN_PT_main_panel,
    AUXIN_PT_commit_panel,
    AUXIN_PT_history_panel,
    AUXIN_PT_remote_panel,
    AUXIN_PT_lock_panel,
    AUXIN_PT_tools_panel,
    AuxinAddonPreferences,
)

def register():
    for cls in classes:
        bpy.utils.register_class(cls)

    bpy.types.Scene.auxin = bpy.props.PointerProperty(type=AuxinSceneProperties)

    # Register handlers
    bpy.app.handlers.save_post.append(auxin_save_handler)
    bpy.app.handlers.load_post.append(auxin_load_handler)

def unregister():
    # Unregister handlers
    if auxin_save_handler in bpy.app.handlers.save_post:
        bpy.app.handlers.save_post.remove(auxin_save_handler)
    if auxin_load_handler in bpy.app.handlers.load_post:
        bpy.app.handlers.load_post.remove(auxin_load_handler)

    del bpy.types.Scene.auxin

    for cls in reversed(classes):
        bpy.utils.unregister_class(cls)

if __name__ == "__main__":
    register()
