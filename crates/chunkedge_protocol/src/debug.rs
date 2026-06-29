//! Packet decode tracing for debugging purposes.
//!
//! Enabled by the `debug-packets` cargo feature. All public items in this
//! module are `#[doc(hidden)]` and considered internal API — they are called
//! exclusively by macro-generated code and the [`super::decode`] entry point.
//!
//! # How it works
//!
//! 1. Before decoding starts, [`enable_packet_recording`] stores the base
//!    pointer of the packet byte slice in thread-local state.
//! 2. For every field, the generated code calls [`log_field_start`], creates an
//!    [`IndentGuard`] (bumping the tree depth), calls the real decoder, and
//!    then calls either [`log_field_success`] or [`log_field_error`].
//! 3. [`log_field_success`] computes the consumed byte range via pointer
//!    arithmetic and pushes a [`Span`] onto the recording context.
//! 4. After decoding finishes (or fails), [`dump_packet_trace`] prints a
//!    colorized hex dump with each byte annotated by which span covers it.

use std::cell::RefCell;
use std::ops::Range;

use owo_colors::OwoColorize;

thread_local! {
    /// Current nesting depth within the decode tree.
    pub static DEPTH: RefCell<usize> = const { RefCell::new(0) };
    static CTX: RefCell<Option<DebugContext>> = const { RefCell::new(None) };
}

struct DebugContext {
    packet_name: String,
    packet_id: i32,
    /// Pointer to byte 0 of the packet being decoded.
    start_ptr: *const u8,
    full_len: usize,
    spans: Vec<Span>,
    roots: Vec<DecodeNode>,
    active_path: Vec<usize>,
}

struct DecodeNode {
    field_name: Option<String>,
    type_name: String,
    value: Option<String>,
    hex_display: Option<String>,
    error: Option<String>,
    error_bytes: Option<String>,
    variant: Option<String>,
    children: Vec<DecodeNode>,
    depth: usize,
}

struct Span {
    range: Range<usize>,
    depth: usize,
    success: bool,
}

/// Initialise the per-thread recording context for `slice`.
pub fn enable_packet_recording(packet_name: &str, packet_id: i32, slice: &[u8]) {
    CTX.with(|c| {
        *c.borrow_mut() = Some(DebugContext {
            packet_name: packet_name.to_owned(),
            packet_id,
            start_ptr: slice.as_ptr(),
            full_len: slice.len(),
            spans: Vec::with_capacity(64),
            roots: Vec::with_capacity(16),
            active_path: Vec::with_capacity(16),
        });
    });
    DEPTH.with(|d| *d.borrow_mut() = 0);
}

/// Close any pending console line and print the hex dump.
pub fn dump_packet_trace(has_error: bool) {
    CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().as_mut() {
            let should_print = if cfg!(feature = "debug-packets") {
                true
            } else if cfg!(feature = "debug-packets-on-error") {
                has_error
            } else {
                false
            };

            if should_print {
                eprintln!(
                    "\n{} {} {}",
                    "▶ Decoding".bold(),
                    ctx.packet_name.cyan().bold(),
                    format!("(ID: 0x{:02x})", ctx.packet_id).dimmed()
                );
                if has_error {
                    eprintln!("{}", "✖ PACKET DECODE FAILED".red().bold());
                }
                for root in &ctx.roots {
                    print_node(root, 0);
                }
                print_hex_dump(ctx);
            }
        }
        *c.borrow_mut() = None;
    });
}

/// RAII guard that increments the decode-tree depth on creation and decrements
/// it on drop.  Created by macro-generated code around every recursive decode
/// call so that child fields are indented beneath their parent.
pub struct IndentGuard;

impl Default for IndentGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl IndentGuard {
    #[must_use]
    pub fn new() -> Self {
        DEPTH.with(|d| *d.borrow_mut() += 1);
        Self
    }
}

impl Drop for IndentGuard {
    fn drop(&mut self) {
        DEPTH.with(|d| *d.borrow_mut() -= 1);
    }
}

/// Compute the byte range that was consumed: from `slice_start[0]` to the
/// first byte of `slice_end` (which is where the reader ended up after the
/// decode).
fn get_range(slice_start: &[u8], slice_end: &[u8]) -> Range<usize> {
    CTX.with(|c| {
        if let Some(ctx) = c.borrow().as_ref() {
            let base = ctx.start_ptr as usize;
            let start = slice_start.as_ptr() as usize - base;
            let consumed = slice_start.len() - slice_end.len();
            start..(start + consumed)
        } else {
            0..0
        }
    })
}

fn clean_type(raw: &str) -> String {
    raw.replace(" < ", "<").replace(" >", ">")
}

fn print_node(node: &DecodeNode, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    let name = node.field_name.as_deref().unwrap_or("?");
    let ct = clean_type(&node.type_name);

    if let Some(err) = &node.error {
        eprintln!(
            "{}{}: {} = {}",
            indent,
            name.red().bold(),
            node.type_name.red(),
            "ERROR".white().on_red()
        );
        eprintln!("{indent}  └─ Reason: {err}");
        if let Some(err_bytes) = &node.error_bytes {
            eprintln!("{indent}  └─ At bytes: {}", err_bytes.red());
        }
    } else {
        let val_str = node.value.as_deref().unwrap_or("?");
        let hex_str = node.hex_display.as_deref().unwrap_or("");

        let hex_suffix = if hex_str.is_empty() {
            String::new()
        } else {
            format!(" [{}]", colorize_by_depth_dimmed(hex_str, node.depth))
        };

        if val_str.contains('\n') {
            eprintln!("{indent}{}: {} ={hex_suffix}", name.bold(), ct.cyan());
            for line in val_str.lines() {
                eprintln!("{}  {}", indent, line.purple());
            }
        } else {
            eprintln!(
                "{indent}{}: {} = {} {hex_suffix}",
                name.bold(),
                ct.cyan(),
                val_str.purple(),
            );
        }

        if let Some(variant) = &node.variant {
            eprintln!("{indent}  └─ {}: {}", "Variant".bold(), variant.yellow());
        }
    }

    for child in &node.children {
        print_node(child, indent_level + 1);
    }
}

/// Print the opening `"  field: Type ..."` line for a field that is about to
/// be decoded.  The line is left open (no newline) until
/// [`log_field_success`] or [`log_field_error`] closes it.
pub fn log_field_start(field_name: Option<&str>, type_name: &str) {
    CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().as_mut() {
            let depth = DEPTH.with(|d| *d.borrow());
            let new_node = DecodeNode {
                field_name: field_name.map(str::to_owned),
                type_name: type_name.to_owned(),
                value: None,
                hex_display: None,
                error: None,
                error_bytes: None,
                variant: None,
                children: Vec::new(),
                depth,
            };

            if ctx.active_path.is_empty() {
                ctx.roots.push(new_node);
                ctx.active_path.push(ctx.roots.len() - 1);
            } else {
                let mut node = &mut ctx.roots[ctx.active_path[0]];
                for &idx in &ctx.active_path[1..] {
                    node = &mut node.children[idx];
                }
                node.children.push(new_node);
                ctx.active_path.push(node.children.len() - 1);
            }
        }
    });
}

/// Print a variant selection line (for enum decoding).
pub fn log_variant(name: &str) {
    CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().as_mut() {
            if let Some(&active_idx) = ctx.active_path.last() {
                let node = if ctx.active_path.len() == 1 {
                    &mut ctx.roots[active_idx]
                } else {
                    let mut n = &mut ctx.roots[ctx.active_path[0]];
                    for &idx in &ctx.active_path[1..ctx.active_path.len() - 1] {
                        n = &mut n.children[idx];
                    }
                    &mut n.children[active_idx]
                };
                node.variant = Some(name.to_owned());
            }
        }
    });
}

/// Record a successfully decoded field, close its console line, and push a
/// [`Span`] into the context.
pub fn log_field_success(
    type_name: &str,
    val: &dyn std::fmt::Debug,
    start_slice: &[u8],
    end_slice: &[u8],
) {
    let range = get_range(start_slice, end_slice);
    let bytes = &start_slice[..range.len()];

    CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().as_mut() {
            let depth = DEPTH.with(|d| *d.borrow());

            ctx.spans.push(Span {
                range,
                depth,
                success: true,
            });

            let val_single_line = format!("{val:?}");
            let val_display = if val_single_line.len() <= 60 {
                val_single_line
            } else {
                let val_pretty = format!("{val:#?}");
                if val_pretty.len() > 500 {
                    if val_pretty.contains('\n') {
                        let ct = clean_type(type_name);
                        format!("{ct} {{ ... }}")
                    } else {
                        format!("{}...", &val_pretty[..497])
                    }
                } else {
                    val_pretty
                }
            };

            let hex_display = if bytes.len() > 12 {
                format!(
                    "{:02x} .. {:02x} ({}b)",
                    bytes[0],
                    bytes[bytes.len() - 1],
                    bytes.len()
                )
            } else if bytes.is_empty() {
                String::new()
            } else {
                bytes
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            };

            if let Some(active_idx) = ctx.active_path.pop() {
                let node = if ctx.active_path.is_empty() {
                    &mut ctx.roots[active_idx]
                } else {
                    let mut n = &mut ctx.roots[ctx.active_path[0]];
                    for &idx in &ctx.active_path[1..] {
                        n = &mut n.children[idx];
                    }
                    &mut n.children[active_idx]
                };
                node.value = Some(val_display);
                node.hex_display = Some(hex_display);
            }
        }
    });
}

/// Record a failed field decode, close any pending console line, and push a
/// failure [`Span`].
pub fn log_field_error(err: &anyhow::Error, start_slice: &[u8]) {
    let range = get_range(start_slice, start_slice);

    CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().as_mut() {
            let depth = DEPTH.with(|d| *d.borrow());

            ctx.spans.push(Span {
                range: range.start..std::cmp::min(range.start + 1, ctx.full_len),
                depth,
                success: false,
            });

            let peek_len = std::cmp::min(start_slice.len(), 32);
            let hex = start_slice[..peek_len]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ");

            if let Some(active_idx) = ctx.active_path.pop() {
                let node = if ctx.active_path.is_empty() {
                    &mut ctx.roots[active_idx]
                } else {
                    let mut n = &mut ctx.roots[ctx.active_path[0]];
                    for &idx in &ctx.active_path[1..] {
                        n = &mut n.children[idx];
                    }
                    &mut n.children[active_idx]
                };
                node.error = Some(err.to_string());
                node.error_bytes = Some(hex);
            }
        }
    });
}

fn print_hex_dump(ctx: &DebugContext) {
    eprintln!("\n{}", " Packet Hex Dump ".black().on_white().bold());

    // SAFETY: the slice is still valid — it lives in the caller's stack frame
    // (PacketFrame::decode) which hasn't returned yet when dump_packet_trace
    // is called.
    let buffer = unsafe { std::slice::from_raw_parts(ctx.start_ptr, ctx.full_len) };

    for (i, chunk) in buffer.chunks(16).enumerate() {
        eprint!("{:04x}: ", i * 16);

        for (j, byte) in chunk.iter().enumerate() {
            let pos = i * 16 + j;

            // Use the deepest span that covers this byte position.
            let active_span = ctx
                .spans
                .iter()
                .filter(|s| s.range.contains(&pos))
                .max_by_key(|s| s.depth);

            let str_byte = format!("{byte:02x}");

            if let Some(span) = active_span {
                if !span.success {
                    eprint!("{} ", str_byte.on_red());
                } else {
                    eprint!("{} ", colorize_by_depth(&str_byte, span.depth));
                }
            } else {
                eprint!("{} ", str_byte.white().dimmed());
            }
        }

        if chunk.len() < 16 {
            eprint!("{}", "   ".repeat(16 - chunk.len()));
        }

        eprint!(" | ");
        for byte in chunk {
            let c = *byte as char;
            if c.is_ascii_graphic() || c == ' ' {
                eprint!("{c}");
            } else {
                eprint!(".");
            }
        }
        eprintln!();
    }
    eprintln!();
}

/// Map a span depth to a fixed palette of six colours, cycling if deeper.
fn colorize_by_depth(s: &str, depth: usize) -> String {
    match depth % 6 {
        0 => s.cyan().to_string(),
        1 => s.green().to_string(),
        2 => s.yellow().to_string(),
        3 => s.blue().to_string(),
        4 => s.magenta().to_string(),
        _ => s.red().to_string(),
    }
}

/// Map a span depth to a dimmed version of six colours, cycling if deeper.
fn colorize_by_depth_dimmed(s: &str, depth: usize) -> String {
    match depth % 6 {
        0 => s.cyan().dimmed().to_string(),
        1 => s.green().dimmed().to_string(),
        2 => s.yellow().dimmed().to_string(),
        3 => s.blue().dimmed().to_string(),
        4 => s.magenta().dimmed().to_string(),
        _ => s.red().dimmed().to_string(),
    }
}
