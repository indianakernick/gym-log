#!/bin/zsh

# brew install librsvg pngcrush svgo

svgo --multipass -o public/favicon.svg -i icon/icon.svg

function rasterize {
  rsvg-convert -h $2 -o public/$1-$2.png icon/icon.svg
  pngcrush -brute -reduce -rem alla -ow public/$1-$2.png
}

for size in 32 64 128; do
  rasterize favicon $size
done

# https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html
for size in 152 167 180; do
  rasterize apple-touch-icon $size
done
