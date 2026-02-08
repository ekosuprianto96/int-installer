# Manifest Specification

This document describes the complete format of the `manifest.json` file for INT packages.

## Basic Format

```json
{
  "version": "1.0",
  "name": "string",
  "package_version": "string",
  "install_scope": "user|system",
  "install_path": "string"
}
```

## Field Reference

### Required Fields

#### `version`
- **Type**: String
- **Required**: Yes (default: "1.0")
- **Description**: Manifest format version
- **Example**: `"1.0"`

#### `name`
- **Type**: String
- **Required**: Yes
- **Description**: Unique package name (identifier)
- **Constraints**: Only alphanumeric, `-`, and `_`
- **Example**: `"my-app"`, `"web_server"`

#### `package_version`
- **Type**: String
- **Required**: Yes
- **Description**: Package version (semver recommended)
- **Example**: `"1.0.0"`, `"2.3.1-beta"`

#### `install_scope`
- **Type**: Enum
- **Required**: Yes
- **Values**: `"user"` or `"system"`
- **Description**: 
  - `"user"`: Install to `~/.local/share/`
  - `"system"`: Install to `/opt/` or `/usr/local/` (requires sudo)

#### `install_path`
- **Type**: String (absolute path)
- **Required**: Yes
- **Description**: Installation target path
- **Constraints**: Must be absolute path, no path traversal
- **Example**: 
  - User: `"/home/user/.local/share/myapp"`
  - System: `"/opt/myapp"`

### Optional Fields

#### `display_name`
- **Type**: String
- **Required**: No
- **Description**: User-friendly display name
- **Default**: Uses `name`
- **Example**: `"My Awesome Application"`

#### `description`
- **Type**: String
- **Required**: No
- **Description**: Short description of the application
- **Example**: `"A powerful web server for Linux"`

#### `author`
- **Type**: String
- **Required**: No
- **Description**: Author or vendor name
- **Example**: `"Acme Corporation"`

#### `entry`
- **Type**: String
- **Required**: No (required for desktop apps)
- **Description**: Name of the main executable (relative to `install_path/bin/`)
- **Example**: `"myapp"`, `"myapp-gui"`

#### `service`
- **Type**: Boolean
- **Required**: No
- **Default**: `false`
- **Description**: Whether to install as a systemd service

#### `service_name`
- **Type**: String
- **Required**: No
- **Default**: Uses `name`
- **Description**: Name of the systemd service
- **Example**: `"my-app"`, `"webserver"`

#### `post_install`
- **Type**: String (relative path)
- **Required**: No
- **Description**: Path to the post-install script
- **Constraints**: Must be relative, no path traversal
- **Example**: `"scripts/install.sh"`

#### `pre_uninstall`
- **Type**: String (relative path)
- **Required**: No
- **Description**: Path to the pre-uninstall script
- **Example**: `"scripts/uninstall.sh"`

#### `desktop`
- **Type**: Object
- **Required**: No (required for GUI apps)
- **Description**: Desktop entry configuration
- **See**: [Desktop Entry Object](#desktop-entry-object)

#### `dependencies`
- **Type**: Array of Objects
- **Required**: No
- **Description**: List of dependencies
- **See**: [Dependency Object](#dependency-object)

#### `required_space`
- **Type**: Number (bytes)
- **Required**: No
- **Description**: Minimum disk space required
- **Example**: `10000000` (10 MB)

#### `architecture`
- **Type**: String
- **Required**: No
- **Description**: Target architecture
- **Values**: `"x86_64"`, `"aarch64"`, `"armv7"`, etc.
- **Example**: `"x86_64"`

#### `license`
- **Type**: String
- **Required**: No
- **Description**: License identifier
- **Example**: `"MIT"`, `"Apache-2.0"`, `"GPL-3.0"`

#### `homepage`
- **Type**: String (URL)
- **Required**: No
- **Description**: Project homepage URL
- **Example**: `"https://example.com"`

## Nested Objects

### Desktop Entry Object

```json
{
  "desktop": {
    "categories": ["Category1", "Category2"],
    "mime_types": ["application/x-custom"],
    "icon": "app-icon",
    "show_in_menu": true,
    "keywords": ["keyword1", "keyword2"]
  }
}
```

#### Fields

- **`categories`** (Array of String): Desktop categories
  - Example: `["Development", "IDE"]`
  - See: https://specifications.freedesktop.org/menu-spec/latest/apa.html
  
- **`mime_types`** (Array of String): Handled MIME types
  - Example: `["text/x-python", "application/x-python"]`
  
- **`icon`** (String): Icon name or path
  - Example: `"myapp"` (theme icon) or `"/path/to/icon.png"`
  
- **`show_in_menu`** (Boolean): Show in application menu
  - Default: `true`
  
- **`keywords`** (Array of String): Keywords for searching
  - Example: `["editor", "code", "programming"]`

### Dependency Object

```json
{
  "dependencies": [
    {
      "name": "docker",
      "min_version": "20.10",
      "check_command": "which docker"
    }
  ]
}
```

#### Fields

- **`name`** (String, Required): Dependency name
- **`min_version`** (String, Optional): Minimum version
- **`check_command`** (String, Optional): Command to check availability

## Complete Examples

### Desktop Application

```json
{
  "version": "1.0",
  "name": "myeditor",
  "display_name": "My Text Editor",
  "package_version": "2.1.0",
  "description": "A modern text editor for developers",
  "author": "John Doe",
  "install_scope": "user",
  "install_path": "/home/user/.local/share/myeditor",
  "entry": "myeditor",
  "service": false,
  "post_install": "scripts/setup.sh",
  "desktop": {
    "categories": ["Development", "TextEditor"],
    "mime_types": ["text/plain", "text/x-python"],
    "icon": "myeditor",
    "show_in_menu": true,
    "keywords": ["editor", "text", "code"]
  },
  "dependencies": [
    {
      "name": "gtk3",
      "min_version": "3.24"
    }
  ],
  "required_space": 50000000,
  "architecture": "x86_64",
  "license": "MIT",
  "homepage": "https://myeditor.example.com"
}
```

### System Service

```json
{
  "version": "1.0",
  "name": "web-server",
  "display_name": "Web Server Pro",
  "package_version": "1.5.2",
  "description": "High-performance web server",
  "author": "Server Solutions Inc",
  "install_scope": "system",
  "install_path": "/opt/web-server",
  "entry": "webserver",
  "service": true,
  "service_name": "web-server",
  "post_install": "scripts/configure.sh",
  "pre_uninstall": "scripts/backup.sh",
  "dependencies": [
    {
      "name": "openssl",
      "min_version": "1.1",
      "check_command": "openssl version"
    }
  ],
  "required_space": 100000000,
  "architecture": "x86_64",
  "license": "Apache-2.0",
  "homepage": "https://webserver.example.com"
}
```

## Validation Rules

1. **Required Fields**: `version`, `name`, `package_version`, `install_scope`, `install_path` must be present
2. **Package Name**: Only alphanumeric, `-`, `_`
3. **Install Path**: Must be an absolute path
4. **Script Paths**: Must be relative paths, `..` is not allowed
5. **Service**: If `service: true`, a `.service` file must exist in `services/`
6. **Desktop**: If there is a desktop config, the `entry` field must be present

## Error Handling

If the manifest is invalid, the installer will reject the package with an error:

- `ManifestParseError`: Invalid JSON
- `MissingField`: Required field is missing
- `ValidationError`: Field does not meet constraints
- `PathTraversalAttempt`: Path contains `..`
- `UnsupportedVersion`: Unsupported manifest version

## Best Practices

1. **Use Semver**: Package version should follow semantic versioning
2. **Descriptive Names**: Use clear and descriptive names
3. **Complete Desktop Entry**: For GUI apps, fill in all desktop fields
4. **Specify Dependencies**: List all required dependencies
5. **Set Required Space**: Help users with disk space information
6. **Use Categories**: Use standard freedesktop.org categories
7. **Test Manifest**: Validate with `int-pack validate manifest.json`

## Schema Validation

JSON Schema is available in `schemas/manifest.schema.json` for automatic validation.

Validate with:

```bash
int-pack validate manifest.json
```

## Versioning

Current version: **1.0**

Breaking changes will use a major version bump (2.0, 3.0, etc.).
