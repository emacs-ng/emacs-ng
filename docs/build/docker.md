# Build emacs-ng using a Docker container

To ease the job of building emacs-ng, we are preparing Docker images
to allow a fully containerized build without installing any build dependencies.

Building this will create a Docker image of about ~9 gb and will take
quite some time (~45 minutes, depending on your specs). Once the
build is completed, you can extract the artifact out of the container
and install it.

Usage example of the Dockerfile for building for Debian GNU/Linux:

Clone the emacs-ng repository and build the image locally:
``` sh
$ git clone --depth=1 https://github.com/emacs-ng/emacs-ng.git
$ cd emacs-ng
$ docker build -t emacs-ng:builder -f docker/Dockerfile.debian .
```

At the end you should read this output:
``` sh
**********************************************************************

 Done. The new package has been saved to

 /src/emacs-ng_0.1-1_amd64.deb
 You can install it in your system anytime using:

      dpkg -i emacs-ng_0.1-1_amd64.deb

**********************************************************************
```

Copy the .deb out of the container, then destroy the container:
``` sh
$ docker run -d --rm --name delete-me emacs-ng:builder bash -c 'tail -f /dev/null' --stop-signal SIGKILL
$ docker cp delete-me:/src/emacs-ng_0.1-1_amd64.deb ~/tmp/
$ docker stop delete-me
```
