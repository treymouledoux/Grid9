<img src=".github/assets/banner.png">

Grid9 is an esoteric interpreted language made in Nim based on a 3x3 grid of zeros and ones.

> [!IMPORTANT]
> This project is activly being re-written in Rust [here](https://github.com/treymouledoux/Grid9/tree/rust).


[![Nightly Release](https://github.com/treymouledoux/Grid9/actions/workflows/nightly.yml/badge.svg)](https://github.com/treymouledoux/Grid9/actions/workflows/nightly.yml)

## Installation

### Binaries

Download the latest release for your platform from the releases page.

### Self Build

#### Windows

Make sure [git](https://github.com/git/git) is installed then run this command.

```powershell
powershell.exe $code = Invoke-RestMethod "https://raw.githubusercontent.com/treymouledoux/Grid9/main/scripts/build_and_install_windows.ps1"; foreach($a in $code) {iex $a;}
```

#### Linux

Make sure [curl](https://github.com/curl/curl) is installed then run this command. It will automatically install [git](https://github.com/git/git) for you.

```bash
curl -s https://raw.githubusercontent.com/treymouledoux/Grid9/main/scripts/build_and_install_linux.sh | bash
```

### Online

You can [run this project in gitpod](https://gitpod.io/#https://github.com/treymouledoux/Grid9) if you don't want to run it locally.

## Documentation

The docs are accessible by running "grid9 d" or "grid9 documentation" in the terminal or by going to the online documentation [here](https://treymouledoux.github.io/Grid9/).

## Contributing

Pull requests are welcome. For major changes, please [open an issue](https://github.com/treymouledoux/Grid9/issues/new) first to discuss what you would like to change.

## License

[gpl-3.0](https://choosealicense.com/licenses/lgpl-3.0/)

## Credits

This project was inspired by the [BrainFuck](https://esolangs.org/wiki/Brainfuck) project on esolangs.org.
