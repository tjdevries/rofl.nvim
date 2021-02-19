# Comparisons

- It seems like a lot of other completion frameworks follow:
    - Setup your sources globally
    - Run a bunch of random settings
    - Now press complete buttons
    - Get completions
- I kinda wanna do something like:
    - You can definitely setup global stuff
    - But you can also configure requests on the fly,
        to give you more power of the completions at any moment.
        - tbh, this idea is heavily inspired by the concepts outlined
            in `:help ins-completion`, like how you can press different
            keyboard shortcuts to get different behavior.


```lua
-- For example,
inoremap <c-x><c-f> :lua require('rofl').request {
    sources = {
        file = true,
    }
}

require('rofl').request {
  sources = {
    lsp = {
      types = {
        'functions',
        'classes',
      }
    }
  }
}
```


