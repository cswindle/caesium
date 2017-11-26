# Caesium
Caesium is an alternative crate registry for Rust, allowing crates to be
publshed directly from Cargo. Below are the key features of Caesium:

 - it is designed to fit into different enterprise systems
 - it has a number of areas that modules can implement support for different systems, these are:
    - storage, where the crates can be stored. This currently includes support for the following:
        - local file system
        - upload to Artifactory

The key areas which it does not have compared to crates.io are:

 - Web UI for querying crates
 - Caesium only supports the publish API

## How to use it
There are two parts that are required for Caesium, these are:

 - git repository for storing information which Cargo uses
 - the actual Caesium server (this application)

The following sections will go through how to set each of these up.

### Setting up git index
You will need a new Git repository which needs to contain a config.json file at
the base of the repository. This needs to contain the following information
(note this is subject to change as the interface is still evolving):

```
{
  "dl": "file:///path/to/my/crates/store",
  "api": "http://127.0.0.1:3000"
}

```

The `dl` field is the URL that uploaded crates can be downloaded from, note that
using a file based URL is only sensible if all machines that need to download
crates have access to the same location.

The `api` field provides details of the URL to access the Caesium server.

### Setting up Caesium configuration
Caesium loads registry.toml from the current directory for the configuration, an
example of the configuration is shown below:

```
[registry]
index = "ssh://git@git.server/index.git"

[storage.file]
location = "/path/to/my/crates/store"
```

The `index` field is the URL for the Git index that was setup in the previous
step.

## Configuration guide
### Registry Config - MANDATORY
This just has a single entry for the index, which is mandatory, below is an example:

```
[registry]
index = "ssh://git@git.server/index.git"
```

### Storage Config - MANDATORY
The storage config contains the following options (one of which must be set):

 - [storage.file]
 - [storage.artifactory]

#### File based storage
There is only one key for file based storage, that is the `location` of where
to store the crates. Below is an example:

```
[storage.file]
location = "/crates/storage/path"
```

#### Artifactory based storage
Artifactory includes the following configuration:

 - base_url
 - api_key

Below is an example:

```
[storage.artifactory]
base_url = "https://artifactory.server/caesium"
api_key = "ABSSJKDNAKSNCNUuansiasncsMKA..."
```


### Server config - OPTIONAL
The server config just has one optional field, this allows setting the port that
Caesium sets the server up on (by default this is 3000). Below is an example:

```
[server]
port = 3000
```
