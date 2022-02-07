# Shove

> Stow, but angry

Shove is a tool similar to [GNU Stow][stow], although Shove is primarily
designed to install [dotfiles][dotfiles]. The main difference from Stow is that
Shove allows more flexible dotfile installation through the optional use of
directories instead of the traditional approach of Stow, in which directories
are installed as symbolic links. Another major difference is how Shove will
remove files and how it avoid users from accidentally removing wrong files.

## How It Works

When executed, the first thing Shove will do is look up for a file named
`.shove.toml` located at the current working directory. This file is the
configuration file for Shove and it must have information about how the program
will install (shove) or uninstall (unshove) dotfiles. In order to know more
about Shove configuration, see the [Configurations](#configurations) topic.

After reading the configuration file, Shove will read the arguments passed via
command line. Some arguments will override settings specified in the
configuration file. For more information about the command line interface, see
the [CLI](#cli) topic.

## Configurations

As previously informed, the configurations for Shove are defined in a file
named `.shove.toml`, which must be written using the [toml][toml] syntax. The
following subtopics covers each configuration that may be set in the
configuration file.

### `absolute`

- Type: Boolean
- Default: `false`

If it's true, created symlinks will have absolute paths, otherwise it will
have relative ones.

### `berserker`

- Type: Boolean
- Default: `false`

If it's true, the code execution will continue on passable errors, otherwise
the execution will finish with exit code 1.

### `depth`

- Type: Unsigned Integer
- Default: `0`

Maximum directory depth to shove. Dotfiles which are directories located at the
maximum depth will be shoved as symlinks instead of directories. Setting this
configuration to zero means that there is no maximum directory depth, which
causes all directories to be shoved as directories themselves.

### `dots`

- Type: Table
- Default: `{}`

A table in which keys stands for name of dots and its values may be strings or
tables. If the value of a key is a string, then its key will be interpreted as
path to the directory containing the dotfiles and its value will be considered
as the destination path where the dotfiles will be shoved. Otherwise if the
value of a key is a table, it must contain two fields which are `src` and
`dest`. `src` must be a string which stands for the source path where is
located the directory containing the dotfiles. `dest` must also be a string
that stands for the path where the dotfiles will be shoved.

A basic shell expansion will be performed in the destination path string in
order to evaluate the actual path.

```toml
[dots]
# Bash dotfiles from `bash` directory with destination at the home directory.
bash = '$HOME'
# Neovim dotfiles from `neovim` directory with destination at the default
# configuration directory.
vi = {src = 'neovim', dest = '~/.config/nvim'}
```

### `follow`

- Type: Boolean
- Default: `false`

If it's true, symlinks located among the dotfiles will be followed, otherwise
they won't be.

### `ignore`

- Type: List of Strings
- Default: `[]`

List of regex strings to match against name of dotfiles. Names matched this way
won't be shoved or unshoved.

```toml
# Ignore all files with the ".unsafe" suffix.
ignore = ['\.unsafe']
```

## CLI

[dotfiles]: https://en.wikipedia.org/wiki/Hidden_file_and_hidden_directory#Unix_and_Unix-like_environments
[stow]: https://www.gnu.org/software/stow
[toml]: https://toml.io
