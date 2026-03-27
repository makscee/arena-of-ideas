set shell := ["bash", "-cu"]

[private]
default:
  just --list

addiza:
    spacetime server add --url https://game-server.izaforge.com iza-web

pingiza:
    spacetime server ping iza-web

publishiza:
    spacetime publish -p server -s iza-web aoi-dev --delete-data -y

gen-binds:
    spacetime generate --lang rust --out-dir client/src/stdb --module-path server -y
