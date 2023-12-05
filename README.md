# Arena of Ideas

publish stdb:
spacetime publish -c aoi -p server

start stdb local:
spacetime start --listen-addr 127.0.0.1:3001

regenerate bindings:
spacetime generate --lang rust --out-dir src/module_bindings --project-path server