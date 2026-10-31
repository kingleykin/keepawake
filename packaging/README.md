# Packaging

Package manager files for KeepAwake. Each one points at the files of a GitHub release, so update the version and checksums after every release.

Get the checksums of a release (drop the `sha256:` prefix when you copy them):

```bash
gh api repos/kingleykin/keepawake/releases/tags/v1.1.0 --jq '.assets[] | .name + " " + .digest'
```

## Homebrew (macOS)

`homebrew/keepawake.rb` is the cask in our own tap, [kingleykin/homebrew-tap](https://github.com/kingleykin/homebrew-tap). After a release, update it here and copy it to `Casks/keepawake.rb` there. Users then install with:

```bash
brew install --cask kingleykin/tap/keepawake
```

The main Homebrew repository needs apps that are signed and notarized by Apple, so the cask lives in our own tap for now.

Checks, run inside the tap:

```bash
brew style Casks/keepawake.rb
brew audit --cask --online kingleykin/tap/keepawake
```

## winget (Windows)

`winget/` has the three manifest files for [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs). Put them in `manifests/k/kingleykin/KeepAwake/<version>/` and open a pull request. On Windows you can check them first. Installing from local manifests has to be allowed once, from an administrator terminal:

```powershell
winget settings --enable LocalManifestFiles
winget validate --manifest packaging\winget
winget install --manifest packaging\winget
```

Once the package is in winget-pkgs, [wingetcreate](https://github.com/microsoft/winget-create) does later updates and their pull requests in one step. It needs a GitHub token, saved once with `wingetcreate token --store`:

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
git push origin HEAD:master           # the AUR only accepts the master branch
```

The in-app **Install** button cannot update a pacman install. AUR users update through pacman or their AUR helper.
