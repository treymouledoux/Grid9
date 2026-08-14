# Package

version       = "2026.001"
author        = "treymouledoux"
description   = "A esoteric interpreted language made in Nim based on a 3x3 grid of zeros and ones. "
license       = "GPL-3.0"
srcDir        = "src"


# Dependencies

requires "nim >= 1.4.0"
requires "docopt >= 0.6.8"
requires "regex >= 0.19.0"
requires "parsetoml >= 1.0.0"


# Tasks

task build, "Build project":
  exec "nimble c --d:release src/main.nim"