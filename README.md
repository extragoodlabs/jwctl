<div align="center">

[![JumpWire](./images/jumpwire-logo.png)](https://jumpwire.io)

#### jwctl - CLI for JumpWire

<!-- Badges - Start -->
[![GitHub Release](https://img.shields.io/github/v/release/extragoodlabs/jwctl?style=flat-square&label=version)](https://github.com/extragoodlabs/jwctl/releases/latest)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/extragoodlabs/jwctl/shipit.yaml?style=flat-square&label=CI)
<!-- Badges - End -->

jwctl is a CLI for interacting with the [JumpWire](https://github.com/extragoodlabs/jumpwire) database gateway.

</div>

## Installation

[Download the release](https://github.com/extragoodlabs/jwctl/releases/latest) for your operating system and extract it to a directory in your path. For example:
```bash
tar xzf jwctl-aarch64-apple-darwin.tar.gz
mv ./jwctl /usr/local/bin
jwctl --version
```

## Authentication

Most commands that interact with a JumpWire server will require the CLI to be authenticated. This is done with a token that can be set in a few different ways.

For one-off or scripted commands, the token can be set as either a CLI argument or environment variable.

```bash
# These formats are equivalent

jwctl -t mysecrettoken <COMMAND>

JW_TOKEN=mysecrettoken jwctl <COMMAND>
```

The token can also be persisted to disk for future use. `jwctl` will attempt to load a token from `~/.config/jwctl/.token`. The token file can be created using the token command:

```bash
JW_TOKEN=mysecrettoken jwctl token set
jwctl <COMMAND>
```

## Configuration

The following sources are loaded and merged together for setting configuration options. Later sources will take precedence when there are conflicts:

- A configuration file at `~/.config/jwctl/config.yaml`
- Environment variables. See below for details.
- Command line flags

### Environment variables

All configuration options can be set using environment variables. Each variable is prefixed with `JW_`. For example, the remote JumpWire URL can be configured by setting `JW_URL`.

### Configuration options

| option | required? | description | examples |
| --- | --- | --- | --- |
| `url` | y | URL of the JumpWire gateway | `jwctl -u <URL> <COMMAND>`, `JW_URL=<URL> jwctl <COMMAND>` |
| `token` | n | Bearer token for authentication | `jwctl -t <TOKEN> <COMMAND>`, `JW_TOKEN=<TOKEN> jwctl <COMMAND>` |

### Configuration file

The url of the JumpWire gateway can also be set via a yaml file. This file needs to be saved under the user home directory: `$HOME/.config/jwctl/config.yaml`

```yaml
# $HOME/.config/jwctl/config.yaml
url: <URL>
```

To persist the auth token to a local file, see the section above describing [authentication](#authentication).

## Commands

### `help`

Print a help message listing all commands. Can be passed a command to print the help message for a specific command.

### `config get`

Print the final configuration after merging together all configuration sources.

### `token set`

Store an authentication token to a persisted configuration file.

### `token whoami`

Check the token against the remote server. If it is valid, the associated permissions will be printed.

### `token generate`

Generate a new token. Permissions must be passed in as pairs of method-action.

#### Example:

```bash
jwctl token generate get:token get:status
```

### `status`

Retrieve the status of the remote server and print it.

### `ping`

Run a ping command to the remote server and print the response.

### `auth list`

List all SSO identity providers configured on the JumpWire proxy server.

### `auth login`

Start a login flow with an SSO provider.

### `db list <type>`

List all databases of a given type. Currently supported types are `postgresql` and `mysql`.

### `db login <token>`

Approve an authentication attempt to a proxied database. The token is generated automatically when connecting a database client to the JumpWire enginer without explicitly setting a password.

If the passed token is valid, jwctl will display a prompt to select which upstream database the proxy should connect to.
