set shell := ["powershell", "-NoProfile", "-Command"]

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
    spacetime generate --lang rust --out-dir client/src/stdb --project-path server -y
