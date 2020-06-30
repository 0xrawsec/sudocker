# Description

Sudocker allows one to restrict which user can run which docker
command without the user being member of the docker group.

# Motivation

Docker is very powerful and useful but giving docker group membership
to regular users is very risky. The reason is that being member of docker
group is equivalent of being root on the machine hosting docker. I would 
qualify this as a security concern since it opens the door to easy priviledge
escalation, but docker team prefers to say that it is intented. So, if like me 
you would like to run docker containers from regular user account safely, you should 
take a look to this project and why not installing the tool.

There is a non-exhaustive list of ways to get root on your docker host using
docker containers, assuming you can issue docker commands without restrictions.
    - using one of the mount overlay option to mount your own root filesystem
    inside a container on which you are root. You can then modify of your host
    with the root user available in the container.
    - using the --privileged options provided by several docker subcommands: https://github.com/moby/moby/issues/9976

# Installation

```
make
sudo make install
```

# Configuration

Edit configuration file `/etc/sudocker/sudockers.toml`

## Example
### Basic

```
[policies]
# john user is only allowed to run docker ps command
john = [
    'docker ps',
]
```

### Advanced 

The configuration file also supports regex so it is easier to allow
groups of commands.

```
[policies]
admin = [
    # !!! Doing this is not recommended
    # allow any docker command (equivalent of being part of docker group)
    # except that the user is not part of docker group, so he cannot access
    # docker socket directly.
    'docker .*?',
]

john = [
    # can run any docker ps command
    'docker ps\s*?.*',
    # can run alpine container only with some options
    'docker run ((--rm|-i|-t) )*alpine .*',
]
```

# Proposed Workflow

1. build images and create containers from a privileged account
2. interact with containers in a restricted manner with the help of sudocker

# Issues

Open issues on this github project