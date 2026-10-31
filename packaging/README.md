# Packaging

Package manager files for KeepAwake. Each one points at the files of a GitHub release, so update the version and checksums after every release.

Get the checksums of a release:

```bash
gh api repos/kingleykin/keepawake/releases/tags/v1.1.0 --jq '.assets[] | .name + " " + .digest'
```

## Homebrew (macOS)

`homebrew/keepawake.rb` is a cask for our own tap, the repository `kingleykin/homebrew-tap`. Copy it to `Casks/keepawake.rb` there. Users then install with:

```bash
brew install --cask kingleykin/tap/keepawake
```

The main Homebrew repository needs apps that are signed and notarized by Apple, so the cask lives in our own tap for now.

Checks, run inside the tap:

```bash
brew style --fix Casks/keepawake.rb
brew audit --cask --online kingleykin/tap/keepawake
```

## winget (Windows)

`winget/` has the three manifest files for [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs). Put them in `manifests/k/kingleykin/KeepAwake/<version>/` and open a pull request. On Windows you can check them first:

```powershell
winget validate --manifest packaging\winget
winget install --manifest packaging\winget
```

For later versions, [wingetcreate](https://github.com/microsoft/winget-create) does the update and the pull request in one step:

```powershell
wingetcreate update kingleykin.KeepAwake --version 1.2.0 --urls https://github.com/kingleykin/keepawake/releases/download/v1.2.0/KeepAwake_1.2.0_x64-setup.exe --submit
```

## AUR (Arch Linux)

`aur/` has the `PKGBUILD` and `.SRCINFO` for `keepawake-app-bin`. The name `keepawake` is already taken on the AUR by a different project. The package repackages the official `.deb`.

Publish it from an Arch machine with an AUR account and SSH key:

```bash
git clone ssh://aur@aur.archlinux.org/keepawake-app-bin.git
cp packaging/aur/PKGBUILD keepawake-app-bin/
cd keepawake-app-bin
makepkg -si                          # build and install it once to test
makepkg --printsrcinfo > .SRCINFO    # always regenerate after editing PKGBUILD
git add PKGBUILD .SRCINFO
git commit -m "keepawake-app-bin 1.1.0"
git push
```

The in-app **Install** button cannot update a pacman install. AUR users update through pacman or their AUR helper.
