# Contributing to ChunkEdge

Before contributing, please first discuss the change you wish to make. For bugs and features proposals, first open an issue with a detailed description of the problem and the solution you would like to see. This makes sure that your contribution is aligned with the project and avoids wasted effort. Otherwise, you risk that your contribution will be rejected when it is submitted for review.

If there are open issues that are already planned by the maintainers, you can reply to the issue to express your interest in working on it stating that you plan to submit a pull request. This will help avoid duplicate work. If the open issue does not yet contain a proposed solution and requires significant change, please first describe your proposal in the issue before starting to work on it.

For general questions and discussions related to the project, please use [GitHub discussions](https://github.com/ChunkEdge/ChunkEdge/discussions).

To get started with your first contribution to the project, you can check out the [issues tagged with "good first issue"](https://github.com/ChunkEdge/ChunkEdge/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22).

## Use of AI

Use of AI and LLM tools is generally allowed when contributing. However, it is the contributor's responsibility to ensure that the generated content is correct. You need to understand exactly what the generated code does, and why it does it. Submitting vibe-coded code will results in your pull request being rejected and will likely lead to a ban from contributing to the project. You should not expect the maintainers to review your AI-generated code when you do not understand it yourself.

## What version of Rust should I use?

It is generally best to use the same stable version of Rust that is currently being used in the CI of the project. You can find out this version by examining the `.github/workflows/ci.yml` file. Some features (such as `miri`) may require the nightly toolchain.

## Playgrounds

Playgrounds are meant to provide a quick and minimal environment to test out new code or reproduce bugs. Playgrounds are also a great way test out quick ideas. This is the preferred method for providing code samples in issues and pull requests.

To get started with a new playground, copy the template to `playground.rs`.

```bash
cp tools/playground/src/playground.template.rs tools/playground/src/playground.rs
```

Make your changes to `crates/playground/src/playground.rs`. To run it:

```bash
cargo run -p playground # simply run the playground, or
cargo watch -c -x "run -p playground" # run the playground and watch for changes
```

## Automatic Checks

When you submit a pull request, your code will automatically run through clippy, rustfmt, etc. to check for any errors or mistakes. If an error does occur, it must be fixed before the pull request can be merged.

## Code Conventions

Here are some rules you should follow for your code. Generally the goal here is to be consistent with existing code, the standard library, and the Rust ecosystem as a whole. Nonconforming code is not necessarily a blocker for accepting your contribution, but conformance is advised.

These guidelines are intended to complement the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html).

### Top-down Modules

Readers of the module should be able to understand your code by reading it from top to bottom. Whenever [items](https://doc.rust-lang.org/reference/items.html) in your module form a parent-child relationship, the parent should be written above the children. Typically this means that important `pub` items are placed before private implementation details.

For instance, here are three functions. Notice how the definition of `foo` is placed above its dependencies. The parent is `foo` while its children are `bar` and `baz`.

```rust
pub fn foo() {
    bar();
    baz();
}

fn bar() {}

fn baz() {}
```

This guideline applies to types as well.

```rust
pub struct Foo {
    bars: Vec<Bar>,
}

struct Bar {
    // ...
}
```

### Getters and Setters

Getters should not start with a `get_` prefix.

<table>
<tr>
<th>Good</th>
<th>Bad</th>
</tr>
<tr>
<td>

```rust
impl Foo {
    fn bar(&self) -> &Bar { ... }
    fn set_bar(&mut self, bar: Bar) { ... }
}
```

</td>
<td>

```rust
impl Foo {
    fn get_bar(&self) -> &Bar { ... }
    fn set_bar(&mut self, bar: Bar) { ... }
}
```

</td>
</tr>
</table>

See [`SocketAddr`](https://doc.rust-lang.org/stable/std/net/enum.SocketAddr.html) for an example of a standard library type that uses this convention.

Under appropriate circumstances a different naming scheme can be used. [`Command`](https://doc.rust-lang.org/stable/std/process/struct.Command.html) is a standard type that demonstrates this.

If a `bar` field exists and no invariants need to be maintained by the getters and setters, it is usually better to make the `bar` field public.

### Bevy `Message`s

Types intended to be used as messages in [`MessageReader`] and [`MessageWriter`] should end in the `Message` suffix. This is helpful for readers trying to distinguish messages from other types in the program.

<table>
<tr>
<th>Good</th>
<th>Bad</th>
</tr>
<tr>
<td>

```rust
struct CollisionMessage { ... }

fn handle_collisions(mut messages: MessageReader<CollisionMessage>) { ... }
```

</td>
<td>

```rust
struct Collision { ... }

fn handle_collisions(mut messages: MessageReader<Collision>) { ... }
```

</td>
</tr>
</table>

### Specifying Dependencies

When adding a new dependency to a crate, make sure you specify the full semver version.

<table>
<tr>
<th>Good</th>
<th>Bad</th>
</tr>
<tr>
<td>

```toml
[dependencies]
serde_json = "1.0.96"
```

</td>
<td>

```toml
[dependencies]
serde_json = "1"
```

</td>
</tr>
</table>

### Writing Unit Tests

When writing unit tests, unwrap errors instead of returning them. Panicking displays the line and column of the error, which is useful for debugging. This information is lost when the error is returned.

<table>
<tr>
<th>Good</th>
<th>Bad</th>
</tr>
<tr>
<td>

```rust
#[test]
fn my_test() {
    some_fallible_func().unwrap();
}
```

</td>
<td>

```rust
#[test]
fn my_test() -> anyhow::Result<()> {
    some_fallible_func()?;
    // ...
    Ok(())
}
```

</td>
</tr>
</table>

### Documentation

All public items should be documented. Documentation must be written with complete sentences and correct grammar. Consider using [intra-doc links](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html) where appropriate.

### Unit Tests

Unit tests help your contributions last! They ensure that your code works as expected and that it continues to work in the future.

whole-server unit tests can be found in [`/src/tests/`](/src/tests).

### Naming Quantities

Variables intended to hold quantities should be written with the `_count` suffix instead of the `num_` prefix.

<table>
<tr>
<th>Good</th>
<th>Bad</th>
</tr>
<tr>
<td>

```rust
let block_count = ...;
```

</td>
<td>

```rust
let num_blocks = ...;
```

</td>
</tr>
</table>
