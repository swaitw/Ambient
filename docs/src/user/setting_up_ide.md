# Setting up your IDE

Our recommended IDE is Visual Studio Code (VSCode).

## Visual Studio Code (VSCode)

Install [Visual Studio Code](https://code.visualstudio.com/), then install the following plugins:

- [rust-analyzer](https://rust-analyzer.github.io/), as described [here](https://code.visualstudio.com/docs/languages/rust).
- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb). This extension is optional, but enables package launching with F5 and will be used to provide debugging support in the future.

`ambient new` will set up your package for VSCode by default by creating a `.vscode/settings.json` for you.

> Mac users: There is currently a bug which triggers a SIGHUP crash each time you close Ambient, when it's started through VSCode. For a fix and more details, see this issue: https://github.com/AmbientRun/Ambient/issues/909

## Emacs

There are multiple ways to configure Emacs as a Rust IDE. The following assumes you are using [rustic](https://github.com/brotzeit/rustic),
[lsp-mode](https://github.com/emacs-lsp/lsp-mode) and [rust-analyzer](https://rust-analyzer.github.io/) libraries. Robert Krahn provides a [comprehensive guide to configuring Emacs for Rust development](https://robert.kra.hn/posts/rust-emacs-setup/#prerequisites).

Once you have Emacs configured for general Rust development, you need to set some explicit values for Ambient packages. Ambient uses some custom `cargo` configuration values that Emacs and rust-analyzer need to know about. You can manually set these variables with the following `elisp`:

```elisp
  (setq lsp-rust-analyzer-cargo-target "wasm32-wasi"
        lsp-rust-analyzer-cargo-watch-args ["--features" "client server"]
        lsp-rust-features ["client" "server"])
```

Furthermore, you can add a `.dir-locals.el` file to your Ambient package directory that Emacs will pick up and load settings for. This is similar to the `.vscode/settings.json` that is created by default. This is an example `.dir-locals.el` file:

```elisp
((rustic-mode . ((eval . (setq-local lsp-rust-analyzer-cargo-target "wasm32-wasi"))
                 (eval . (setq-local lsp-rust-analyzer-cargo-watch-args ["--features" "client server"]))
                 (eval . (setq-local lsp-rust-features ["client" "server"])))))
```

## Other IDEs

To get rust-analyzer to work, you need to make sure it's building with the `server` and `client` feature flags enabled. See [.vscode/settings.json](https://github.com/AmbientRun/Ambient/blob/main/app/src/cli/package/new_package_template/.vscode/settings.json) for an example.
