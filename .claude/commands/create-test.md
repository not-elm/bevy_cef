---
allowed-tools: Bash(cargo test:*),
description: Create new tests
---

1. Reading $ARGUMENTS and understanding the implementation details.
2. Creating new tests for $ARGUMENTS in `tests` module in the same file.
3. Run `cargo test --workspace --all-features` to ensure all tests pass.
4. If any tests fail, fix the issues and re-run the tests.

## Contracts

- You have to write the rust-doc that describes the test case for each test function.


以下の手順で問題を修正してください。

## 現在発生している問題

以下のようなHTMLにCEFのカスタムリソースハンドラを使って`cef://localhost/brp.html`経由でアクセスしています。
このHTMLからvideoリソースを読み込むと、`cef://localhost/test.mov`というリクエストURLでローカルResourceHandlerのopenメソッドが呼ばれ、response_headers, readメソッドでヘッダーとレスポンスボディが返されることが期待されます。
初回と２回目までのreadメソッドは呼ばれるのですが、３回目以降のreadメソッドが呼ばれず、何度もopenメソッドが呼ばれてしまっているため、正常にレスポンスが返却できるように修正してください。

```htm
<html>
    <body>
        <video controls>
            <source src="test.mov">
        </video>
    </body>
</html>
```

## 手順

1. [CEFのResourceHandler](https://cef-builds.spotifycdn.com/docs/122.0/classCefResourceHandler.html)の仕様を深く読み込み理解する
2. `crates/bevy_cef_core/src/browser_process/localhost.rs`以下にResourceHandlerを使ってローカルリソースを読み込むコードが書いています。深く読み込んで現状の実装を理解してください。
3.  openメソッドが複数回呼ばれ、readメソッドが３回目以降呼ばれない原因を調査する
4. 原因を特定し、修正する