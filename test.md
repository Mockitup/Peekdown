# Markdown Feature Test

This document tests all major Markdown features supported by GFM (GitHub Flavored Markdown).

---

## Headings

### Third Level
#### Fourth Level
##### Fifth Level
###### Sixth Level

---

## Text Formatting

This is **bold text** and this is *italic text* and this is ***bold italic***.

This is ~~strikethrough text~~ and this is `inline code`.

Here's a line with a [hyperlink](https://example.com) and an [email link](mailto:test@example.com).

> This is a blockquote. It can span multiple lines
> and continue like this.
>
> > Nested blockquotes are also supported.
> > They can go multiple levels deep.

---

## Lists

### Unordered List
- First item
- Second item
  - Nested item A
  - Nested item B
    - Deep nested item
- Third item

### Ordered List
1. First step
2. Second step
   1. Sub-step one
   2. Sub-step two
3. Third step

### Task List
- [x] Complete project setup
- [x] Implement core features
- [ ] Add drag and drop
- [ ] Write documentation
- [ ] Deploy to production

---

## Code Blocks

Inline: Use `console.log()` for debugging.

```javascript
// Fibonacci sequence generator
function* fibonacci() {
  let a = 0, b = 1;
  while (true) {
    yield a;
    [a, b] = [b, a + b];
  }
}

const fib = fibonacci();
for (let i = 0; i < 10; i++) {
  console.log(fib.next().value);
}
```

```rust
fn main() {
    let greeting = "Hello from mdview!";
    println!("{}", greeting);

    let numbers: Vec<i32> = (1..=10)
        .filter(|n| n % 2 == 0)
        .collect();

    println!("Even numbers: {:?}", numbers);
}
```

```css
:root {
  --bg-primary: #1e1e2e;
  --accent: #89b4fa;
}

.container {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
  padding: 2rem;
}
```

---

## Tables

| Feature | Status | Priority | Notes |
|---------|:------:|:--------:|-------|
| File Open | Done | High | Ctrl+O |
| File Save | Done | High | Ctrl+S |
| Drag & Drop | Done | Medium | Drop .md files |
| Toggle Preview | Done | High | Ctrl+E |
| Syntax Highlighting | Planned | Low | Future release |
| Split Pane | Planned | Medium | v2.0 |

### Right-Aligned Table

| Metric | Value |
|-------:|------:|
| Binary Size | 620 KB |
| Startup Time | ~50ms |
| Memory Usage | ~15 MB |
| Dependencies | 6 crates |

---

## Horizontal Rules

Three different styles:

---

***

___

---

## Images

![Placeholder Image](https://via.placeholder.com/600x200/1e1e2e/89b4fa?text=mdview+Preview)

---

## Links

- [Regular link](https://example.com)
- [Link with title](https://example.com "Example Website")
- Autolink: https://github.com
- Email: test@example.com

---

## Emphasis Edge Cases

This is *single asterisk italic* and _single underscore italic_.

This is **double asterisk bold** and __double underscore bold__.

This is ***triple asterisk bold italic*** and ___triple underscore bold italic___.

Mixed: **bold with *italic* inside** and *italic with **bold** inside*.

---

## Block Elements

### Paragraphs with Line Breaks

This is the first paragraph. It has multiple sentences. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

This is the second paragraph. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

### Nested Blockquotes

> Level 1 quote
>
> > Level 2 quote
> >
> > > Level 3 quote — going deeper!

### Mixed List Content

1. **Step One**: Initialize the project
   - Run `cargo init`
   - Edit `Cargo.toml`

2. **Step Two**: Build the frontend
   > Note: All HTML/CSS/JS is embedded at compile time

3. **Step Three**: Test everything
   ```bash
   cargo build --release
   ./target/release/mdview.exe test.md
   ```

---

## Special Characters & Escaping

Backslash escapes: \*not italic\* \`not code\` \[not a link\]

HTML entities: &copy; 2026 &mdash; All rights reserved &bull; Built with Rust

Pipes in text: Use | for table columns | like this.

---

## Long Content for Scroll Testing

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus lacinia odio vitae vestibulum vestibulum. Cras venenatis euismod malesuada. Nulla facilisi. Etiam non diam ante. Donec sit amet magna non nunc tristique varius. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas.

Praesent commodo cursus magna, vel scelerisque nisl consectetur et. Donec sed odio dui. Donec ullamcorper nulla non metus auctor fringilla. Vestibulum id ligula porta felis euismod semper. Aenean lacinia bibendum nulla sed consectetur. Cras mattis consectetur purus sit amet fermentum.

Integer posuere erat a ante venenatis dapibus posuere velit aliquet. Cras justo odio, dapibus ut facilisis in, egestas eget quam. Maecenas faucibus mollis interdum. Fusce dapibus, tellus ac cursus commodo, tortor mauris condimentum nibh, ut fermentum massa justo sit amet risus. Nullam quis risus eget urna mollis ornare vel eu leo.

---

*End of test document — if you can read this, mdview is working!*
