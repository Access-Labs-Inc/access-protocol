#!/bin/bash

autoflake --in-place --remove-all-unused-imports ./src/*.py
autopep8 --in-place ./src/*.py
isort ./src/*.py
pycodestyle ./src/*.py