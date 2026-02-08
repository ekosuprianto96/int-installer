# Manifest Specification

Dokumen ini menjelaskan format lengkap file `manifest.json` untuk INT packages.

## Format Dasar

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
- **Description**: Versi format manifest
- **Example**: `"1.0"`

#### `name`
- **Type**: String
- **Required**: Yes
- **Description**: Nama unik package (identifier)
- **Constraints**: Hanya alphanumeric, `-`, dan `_`
- **Example**: `"my-app"`, `"web_server"`

#### `package_version`
- **Type**: String
- **Required**: Yes
- **Description**: Versi package (semver recommended)
- **Example**: `"1.0.0"`, `"2.3.1-beta"`

#### `install_scope`
- **Type**: Enum
- **Required**: Yes
- **Values**: `"user"` atau `"system"`
- **Description**: 
  - `"user"`: Install ke `~/.local/share/`
  - `"system"`: Install ke `/opt/` atau `/usr/local/` (requires sudo)

#### `install_path`
- **Type**: String (absolute path)
- **Required**: Yes
- **Description**: Path tujuan instalasi
- **Constraints**: Must be absolute path, no path traversal
- **Example**: 
  - User: `"/home/user/.local/share/myapp"`
  - System: `"/opt/myapp"`

### Optional Fields

#### `display_name`
- **Type**: String
- **Required**: No
- **Description**: Nama tampilan yang user-friendly
- **Default**: Menggunakan `name`
- **Example**: `"My Awesome Application"`

#### `description`
- **Type**: String
- **Required**: No
- **Description**: Deskripsi singkat aplikasi
- **Example**: `"A powerful web server for Linux"`

#### `author`
- **Type**: String
- **Required**: No
- **Description**: Nama author atau vendor
- **Example**: `"Acme Corporation"`

#### `entry`
- **Type**: String
- **Required**: No (required for desktop apps)
- **Description**: Nama executable utama (relative to `install_path/bin/`)
- **Example**: `"myapp"`, `"myapp-gui"`

#### `service`
- **Type**: Boolean
- **Required**: No
- **Default**: `false`
- **Description**: Apakah install sebagai systemd service

#### `service_name`
- **Type**: String
- **Required**: No
- **Default**: Menggunakan `name`
- **Description**: Nama systemd service
- **Example**: `"my-app"`, `"webserver"`

#### `post_install`
- **Type**: String (relative path)
- **Required**: No
- **Description**: Path ke script post-install
- **Constraints**: Must be relative, no path traversal
- **Example**: `"scripts/install.sh"`

#### `pre_uninstall`
- **Type**: String (relative path)
- **Required**: No
- **Description**: Path ke script pre-uninstall
- **Example**: `"scripts/uninstall.sh"`

#### `desktop`
- **Type**: Object
- **Required**: No (required for GUI apps)
- **Description**: Konfigurasi desktop entry
- **See**: [Desktop Entry Object](#desktop-entry-object)

#### `dependencies`
- **Type**: Array of Objects
- **Required**: No
- **Description**: Daftar dependencies
- **See**: [Dependency Object](#dependency-object)

#### `required_space`
- **Type**: Number (bytes)
- **Required**: No
- **Description**: Minimum disk space yang dibutuhkan
- **Example**: `10000000` (10 MB)

#### `architecture`
- **Type**: String
- **Required**: No
- **Description**: Target architecture
- **Values**: `"x86_64"`, `"aarch64"`, `"armv7"`, dll
- **Example**: `"x86_64"`

#### `license`
- **Type**: String
- **Required**: No
- **Description**: License identifier
- **Example**: `"MIT"`, `"Apache-2.0"`, `"GPL-3.0"`

#### `homepage`
- **Type**: String (URL)
- **Required**: No
- **Description**: URL homepage project
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
  
- **`mime_types`** (Array of String): MIME types yang di-handle
  - Example: `["text/x-python", "application/x-python"]`
  
- **`icon`** (String): Nama icon atau path
  - Example: `"myapp"` (theme icon) atau `"/path/to/icon.png"`
  
- **`show_in_menu`** (Boolean): Tampilkan di application menu
  - Default: `true`
  
- **`keywords`** (Array of String): Keywords untuk search
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

- **`name`** (String, Required): Nama dependency
- **`min_version`** (String, Optional): Versi minimum
- **`check_command`** (String, Optional): Command untuk check ketersediaan

## Contoh Lengkap

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

1. **Required Fields**: `version`, `name`, `package_version`, `install_scope`, `install_path` harus ada
2. **Package Name**: Hanya alphanumeric, `-`, `_`
3. **Install Path**: Harus absolute path
4. **Script Paths**: Harus relative path, tidak boleh `..`
5. **Service**: Jika `service: true`, harus ada file `.service` di `services/`
6. **Desktop**: Jika ada config desktop, harus ada field `entry`

## Error Handling

Jika manifest invalid, installer akan menolak package dengan error:

- `ManifestParseError`: JSON tidak valid
- `MissingField`: Field required tidak ada
- `ValidationError`: Field tidak memenuhi constraints
- `PathTraversalAttempt`: Path mengandung `..`
- `UnsupportedVersion`: Versi manifest tidak didukung

## Best Practices

1. **Gunakan Semver**: Package version mengikuti semantic versioning
2. **Descriptive Names**: Gunakan nama yang jelas dan deskriptif
3. **Complete Desktop Entry**: Untuk GUI apps, lengkapi semua field desktop
4. **Specify Dependencies**: List semua dependencies yang dibutuhkan
5. **Set Required Space**: Bantu user dengan info disk space
6. **Use Categories**: Gunakan standard freedesktop.org categories
7. **Test Manifest**: Validasi dengan `int-pack validate manifest.json`

## Schema Validation

JSON Schema tersedia di `schemas/manifest.schema.json` untuk validasi otomatis.

Validasi dengan:

```bash
int-pack validate manifest.json
```

## Versioning

Current version: **1.0**

Breaking changes akan menggunakan major version bump (2.0, 3.0, dll).
