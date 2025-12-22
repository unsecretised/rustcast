# RUSTCAST EXTENSIONS

## Preamble:

RustCast doesn't support extensions yet.

However, it is in my todo list to support them.

This page is currently about methods that might be used to add extensions to
RustCast.

## Methods:

1. Using an MCP server.
   - MCP Servers are used by GenAI to call functions and retrieve data.
   - Maybe if we stripped the AI from MCP Servers, they could be used for
     extensions, not just in rustcast, but in all projects.

1. Using WASM:
   - The way Zed does their extension support. Maybe I could also use that? 
   - Their article can be found [here](https://zed.dev/blog/zed-decoded-extensions)
