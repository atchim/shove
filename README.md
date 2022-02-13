# Shove

> Stow, but angry

Shove is a tool similar to [GNU Stow], although Shove is primarily
designed to manage configuration files. The main difference from Stow is that
Shove allows more flexible file installation through the optional use of
directories instead of the traditional approach of Stow, in which directories
are installed as symbolic links. Another major difference is how Shove will
uninstall files and how it avoid users from accidentally performing dangerous
removals.

## Terminology

### Dot

A setting defined at the `.shove.toml` configuration file for Shove. This
setting specifies a name for itself, a source path to a directory and a
destination path which, after performed shell expansion, must evaluate to a
path where the source files should be installed. For more information, see
[dots](#dots).

### Dotfile

A file contained in the source directory of a [dot](#dot). This file is taken
as reference by Shove to perform filesystem operations as, for instance,
installing and uninstalling files.

<!--TODO: Create subtopic for "tree" term.-->

## How It Works

When executed, the first thing Shove will do is look up for a file named
`.shove.toml` located at the current working directory. This is the
configuration file for Shove and it may have information about how the program
will manage dotfiles. For more information, see
[Configurations](#configurations).

After reading the configuration file, Shove will read the arguments passed via
command line. Some arguments will override settings specified in the
configuration file. For more information, see [CLI](#cli).

<!--TODO: Continue the explanation.-->

## Configurations

As previously mentioned, the configurations for Shove are defined in a file
named `.shove.toml`, which must be written using the [TOML] syntax. The
following subtopics covers each configuration that may be set in the
configuration file.

### `absolute`

- Type: Boolean
- Default: `false`

If true, when creating symbolic links, their paths will be absolute; otherwise
the paths will be relative.

### `berserker`

- Type: Boolean
- Default: `false`

If true, the code execution will continue on passable errors; otherwise the
execution will finish immediately with exit code 1.

### `depth`

- Type: Unsigned Integer
- Default: `0`

Maximum dotfile depth to manage. The depth is calculated based on the nesting
level of a dotfile relative to the dot source directory. A dotfile which is
child of the source directory of a dot has depth one. A dotfile which is
child of the one previously mentioned has depth two and so on...

Dotfiles which are directories with maximum depth will be installed as symbolic
links instead of directories. Setting this configuration to zero means that
there is no maximum depth, resulting in all directories being installed as
directories themselves.

```toml
# To have a behavior similar to how Stow works, set the depth to 1.
depth = 1
```

### `dots`

- Type: Table
- Default: `{}`

A table in which each key stands for the name of a dot. Each value for a key
may be a string or a table. If the value is a string, then its key will be
considered the dot source path and its value will be considered the destination
path.

Otherwise, if the value of a key is a table, it must contain two fields which
are `src` and `dest`. `src` must be a string and it will be considered as the
dot source path. `dest` must also be a string and it will be considered as the
destination path of the dot. For more information, see [Dot](#dot).

```toml
[dots]
# A dot with name and source path set to "bash" and with destination path set
# to "$HOME".
bash = '$HOME'
# A dot with name set to "vi", source path set to "neovim" and destination path
# set to "~/.config/nvim".
vi = {src = 'neovim', dest = '~/.config/nvim'}
```

### `follow`

- Type: Boolean
- Default: `false`

If true, symbolic links among the dotfiles will be followed; otherwise they
won't be.

### `ignore`

- Type: List of Strings
- Default: `[]`

List of regex strings to match against name of dotfiles. Names matched this way
won't be managed by Shove.

```toml
# Ignore all files with the ".unsafe" suffix.
ignore = ['\.unsafe$']
```

### `rage`

- Type: Unsigned Integer
- Default: `0`

This setting controls how files located at the destination tree will be
removed if necessary. Higher rage level means more permission to perform
dangerous removals.

- With rage level 0, which is the default, Shove will only be able to remove
  symbolic links that refers to a dotfile.
- With rage level 1, Shove will be able to remove symbolic links referring to
  an arbitrary location.
- With rage level 2, Shove will be able to remove common files and empty
  directories.
- With rage level 3 or more, Shove will be able to remove non-empty
  directories.

## CLI

> `shove [-a SWITCH] [-b SWITCH] [-c WHEN] [-d LEVEL] [-f SWITCH] [-n]
> [-q ...] [-r LEVEL] [-u] [-v ...] [DOT ...]`

### Flags

#### `-n`, `--no`

Do not make any change to the filesystem; basically a dry-run.

#### `-q`, `--quiet`

- Cumulative

Decrease output verbosity. Only the first occurrence of this flag will take
effect.

#### `-u`, `--unshove`

Uninstall dotfiles.

#### `-v`, `--verbose`

- Cumulative

Increase output verbosity. Only the first three occurrences of this flag will
take effect.

[GNU Stow]: https://www.gnu.org/software/stow
[TOML]: https://toml.io
