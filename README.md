# wg_activity_notify

![release](https://github.com/sequoiia/wg_activity_notify/actions/workflows/release.yml/badge.svg) ![release](https://github.com/sequoiia/wg_activity_notify/actions/workflows/tagged-release.yml/badge.svg)

Keep track of when clients connect as well as disconnect from your Wireguard server. Optionally send a notification to one or more destination when this happens.

## Supported notification providers

Currently these providers can be used:

- Pushover
- Discord

Is a provider missing that you want to use? Feel free to submit a PR or issue.

## Installation

### Docker

Image repo: https://hub.docker.com/r/sequoiia/wg_activity_notify

Make sure you have a config.yml that can be mounted by the container. Take a look at [config.yml.example](config.yml.example) for an example of how a config.yml could look like.

Both of the Docker examples assumes your Wireguard interfaces are available in the host OS.

TZ environment variable is supported, so make sure to set it to fit your timezone.

#### Docker run

```shell
docker run --rm -it --name wg_activity_notify --net=host --privileged -v config.yml:/app/config.yml sequoiia/wg_activity_notify:latest
```

#### Docker-compose

```yaml
version: '3.7'
services:
  daemon:
    image: "sequoiia/wg_activity_notify:latest"
    init: true
    network_mode: host
    cap_add:
      - NET_ADMIN
    restart: unless-stopped
    environment:
      TZ: 'Europe/Copenhagen'
    volumes:
      - ./config.yml:/app/config.yml
```

### Binary

Currently no binaries are built by CI. You can compile them yourself if you have a Rust toolchain installed.

A simple `cargo install --git https://github.com/SEQUOIIA/wg_activity_notify.git` should compile and install the latest changes from the master branch to your Cargo bin directory(usually $HOME/.cargo/bin). If you wish to install a particular version, use the `--tag` arg.

## Missing features

Currently missing some functionality that I'd like to add at some point:

- Support more than one wg interface
- Add general webhook notification provider(should cover a fair chunk of these kind of services without having to add support for each and every one of them explictly)
