# rofl.nvim

Rust On the FLy completion engine for Neovim.

## Why Rust?

It's 2021. I think the question you should be asking yourself is "Why NOT Rust?!?? (btw)"

## Goals

- [ ] All configuration should be done on the Lua side of things.
    - I will pretend vimL doesn't exist as much as possible at the moment.
    - All completion is done via functions. I don't like variables. They are hard to deal with.
- [ ] Able to add completion sources via Lua
- [ ] Able to add completion sources via Rust (I don't know if this is possible, but it seems cool)
    - [ ] Related to this might be the ability to add things via any remote plugin... idk.
- [ ] Snippets & related expansion
    - I don't know how complicated we'll get with these, but I want to do some stuff with snippets so that I can figure out how we can implement the right stuff in Neovim core :)
- [ ] Builtin sources
    - [ ] Builtin LSP
    - [ ] Buffer
        - [ ] Fuzzy buffer completion with bonus points for being close to the cursor
    - [ ] File
    - [ ] ... your ideas here
    - [ ] Can we bundle neovim, access the C codes for some completion sources and use them "async"-y from neovim itself...?
        - Could be a fun project for messing around with C & Rust interop.
