# TinyGhettoBox

This project is inspired by MuPiBox and aims to implement a more clean and more performant version.
This project does not aim to copy or clone the original Mupibox project, but is heavily inspired by it and
re-uses some knowledge like how to overclock the sd card and how to set the resolution.
Beside that, this project was implemented from scratch with different approaches.

The project utilizes the great performance and low footprint of the language Rust.
The admin interface is implemented with Typescript + React and served from an API server written in Rust.
The user interface is implemented with Rust and Gtk4-rs. While the GTK application still has a considerable
memory footprint of 100-200MB, this is way lower than the Chrome approach of the original Mupibox project.
To optimize further, the playlists are created upfront in the admin interface instead of running a query to the
spotify API on demand. Obviously this has the downside of new entries in Spotify not being available automatically.

### Building

In order to build the project, you need to install rust, gtk4 and librsvg.

To [install gtk4 on windows](https://www.gtk.org/docs/installations/windows) you can
use [gvsbuild](https://github.com/wingtk/gvsbuild).

```
gvsbuild build gtk4 librsvg
```

To run the project with gtk, the cargo command needs following environment variables

```
INCLUDE=C:\gtk-build\gtk\x64\release\include\;C:\gtk-build\gtk\x64\release\include\cairo\;C:\gtk-build\gtk\x64\release\include\glib-2.0\;C:\gtk-build\gtk\x64\release\include\gobject-introspection-1.0\;C:\gtk-build\gtk\x64\release\lib\glib-2.0\include;
LIB=C:\gtk-build\gtk\x64\release\lib
```

In order to cross compile for raspberry pi with dietpi flashed, we need the following:

- docker or podman to build a compilation environment image
    - the environment needs
        - The same debian version as base image (bookworm) to build with the same or compatible glibc version
        - build-essentials to compile host specific files like build.rs which are executed during build time
        - pkg-config to resolve dependencies on the host
        - libglib2.0-dev to get glib-compile-resources binary which is used in our build.rs file
        - debian-archive-keyring to have the keyring in place for building a local debian sysroot
        - multistrap to create a debian sysroot having all packages needed for compilation to target arch
- This image is executed with relevant folders mounted
    - mounting . to project/tinyghettobox to build this project
    - mounting ../../kira to project/kira to have the customized version, which supports remote streaming
    - mounting ~/.cargo to .cargo to have compile cache active

```bash
podman build -t cross_compile .
podman run -ti -v .:/project/tinyghettobox -v ../../kira:/project/kira -v ~/.cargo/:/.cargo cross_compile
```

### Cross compiling

In order to cross compile for Raspberry PI 3b, this project uses a Dockerfile with the same Debian version as the
target (bookworm). We can't use the rust cross utility, because it uses ubuntu xenial as base image. This has a very low
glibc version and
is therefore highly compatible, but this also means the package manager has super outdated packages, and we can't
install gtk4. Building it from source is an enormous effort because all dependencies (gdk, gsk, cairo, pango, etc) has
to be compiled
as well. Our docker image provides a sysroot folder, which mirrors dietpi based on debian bookworm running on the target
system at aarch64/arm64 arch.
To compile the sources first build the image `podman build -t cross_compile .` and then run the image with the project
mounted
`podman run -ti -v .:/project/tinyghettobox -v ../../kira:/project/kira -v ~/.cargo/:/.cargo cross_compile`
