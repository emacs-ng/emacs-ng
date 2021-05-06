#!/bin/sh

# emacs-ng documentation is generated using the Python package mkdocs
# you can install it with `pip install mkdocs`
# and then run `mkdocs build`, it will load `mkdocs.yml`
# generated HTML documentation will be found in `./site`
# connect your browser to http://localhost:8000

# or using a Docker container:
# - create a Github token with one permission: read-packages
# - export it an environment variable (`export GITHUB_TOKEN=xxx`)
# - run this script

GITHUB_ACTOR=YOUR_GITHUB_USERNAME
GITHUB_TOKEN=${GITHUB_TOKEN}

cp -rf images README.md docs
docker login docker.pkg.github.com --username $GITHUB_ACTOR --password $GITHUB_TOKEN
docker run --network host --rm -v ${PWD}:/docs docker.pkg.github.com/emacs-ng/docs-image/docs-image -- serve
