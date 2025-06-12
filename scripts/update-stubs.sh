#!/bin/bash
# Script to update stub version when Cargo.toml version changes

VERSION=$(grep -m1 'version = ' Cargo.toml | cut -d '"' -f2)
sed -i.bak "s/@version [0-9]\+\.[0-9]\+\.[0-9]\+/@version $VERSION/g" nucleation-stubs.php
rm -f nucleation-stubs.php.bak
echo "Updated stubs to version $VERSION"