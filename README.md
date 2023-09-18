# Dup img finder

Find duplicate images by similarity.

## Usage

```bash
dup-img-finder /PATH/TO/IMAGE/DIR
```

Configuration will be generated automatically in `~/dif.toml`

Currently, the following image formats are supported:

* GIF
* JPG (JPEG)
* PNG
* BMP
* WEBP

## Build

```bash
git clone --depth 1 https://github.com/Thaumy/dup-img-finder.git
cd dup-img-finder
cargo build -r
```

## Install over Nix

1. [Enable NUR](https://github.com/nix-community/NUR#installation)

2. Edit `configuration.nix` ：

```nix
environment.systemPackages = with pkgs; [
  nur.repos.thaumy.dup-img-finder
];
```

