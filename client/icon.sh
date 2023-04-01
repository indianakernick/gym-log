#!/bin/zsh

# brew install librsvg pngcrush svgo

svgo --multipass -o public/favicon.svg -i icon/round.svg

function rasterize {
  rsvg-convert -h $3 -o public/$2-$3.png icon/$1.svg
  pngcrush -brute -reduce -rem alla -ow public/$2-$3.png
}

for size in 32 64 128; do
  rasterize round favicon $size
done

# https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html
for size in 152 167 180; do
  rasterize default apple-touch-icon $size
done
