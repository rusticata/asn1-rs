use asn1_rs::{Any, AnyIterator, Class, DerMode, DerParser, Header, Input, Length, Tag};
use colored::*;
// use oid_registry::{format_oid, Oid as DerOid, OidRegistry};
use std::cmp::min;
use std::error::Error;
use std::marker::PhantomData;
use std::{env, fs};

struct Context<'a> {
    // oid_registry: OidRegistry<'a>,
    hex_max: usize,

    /// number of characters required to print an offset in file
    off_width: usize,

    /// Dump header
    dump_header: bool,

    /// Dump object hex data
    dump_hex_data: bool,

    t: PhantomData<&'a ()>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        // let oid_registry = OidRegistry::default().with_all_crypto().with_x509();
        Context {
            // oid_registry,
            hex_max: 64,
            off_width: 0,
            dump_header: false,
            dump_hex_data: false,
            t: PhantomData,
        }
    }
}

fn print_offsets(start: usize, len: usize, ctx: &Context) {
    print!("{start:width$} {len:width$}: ", width = ctx.off_width);
}

fn print_offsets_none(ctx: &Context) {
    print!("{:width$} {:width$}: ", "", "", width = ctx.off_width);
}

fn print_header(header: &Header, depth: usize, _ctx: &Context) {
    let class = match header.class() {
        Class::Universal => "UNIVERSAL".to_string().white(),
        c => c.to_string().cyan(),
    };
    let mut detailed_print = false;

    if header.tag().0 >= 31 || header.class() != Class::Universal {
        detailed_print = true;
    }

    let hdr = if !detailed_print {
        // well-known tag
        header.tag().to_string().to_ascii_uppercase()
    } else {
        format!(
            "[c:{} t:{}({}) l:{}]",
            class,
            header.tag().0,
            header.tag().to_string().white(),
            str_of_length(header.length())
        )
    };
    indent_print!(depth, "{}", hdr);
}

#[macro_export]
macro_rules! indent_print {
    ( $depth: expr, $fmt:expr ) => {
        print!(concat!("{:indent$}",$fmt), "", indent = 2*$depth)
    };
    ( $depth: expr, $fmt:expr, $( $x:expr ),* ) => {
        print!(concat!("{:indent$}",$fmt), "", $($x),*, indent = 2*$depth)
    };
}

#[macro_export]
macro_rules! indent_println {
    ( $depth: expr, $fmt:expr ) => {
        indent_print!($depth, $fmt); println!();
    };
    ( $depth: expr, $fmt:expr, $( $x:expr ),* ) => {
        indent_print!($depth, $fmt, $($x),*); println!();
    };
}

fn print_hex_dump(depth: usize, bytes: &[u8], ctx: &Context) {
    let max_len = ctx.hex_max;
    let m = min(bytes.len(), max_len);
    for chunk in bytes[..m].chunks(16) {
        print_offsets_none(ctx);
        indent_print!(depth, "");
        for b in chunk.iter() {
            print!("{b:02X} ");
        }
        println!();
    }
    if bytes.len() > max_len {
        print_offsets_none(ctx);
        indent_println!(
            depth,
            "... <continued ({} more bytes)>",
            bytes.len() - max_len
        );
    }
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let mut ctx = Context::default();
    for arg in env::args().skip(1) {
        match arg.as_ref() {
            "-h" => {
                ctx.dump_header = true;
                continue;
            }
            "-hh" => {
                ctx.dump_header = true;
                ctx.dump_hex_data = true;
                continue;
            }
            _ => (),
        }
        let filename = arg;
        eprintln!("File: {}", filename);
        let content = fs::read(&filename)?;
        // check for PEM file
        if filename.ends_with(".pem") || content.starts_with(b"----") {
            let pems = pem::parse_many(&content).expect("Parsing PEM failed");
            if pems.is_empty() {
                eprintln!("{}", "No PEM section decoded".bright_red());
                continue;
            }
            for (idx, pem) in pems.iter().enumerate() {
                eprintln!("Pem entry {} [{}]", idx, pem.tag().bright_blue());
                ctx.off_width = (pem.contents().len() as f32).log10().floor() as usize + 1;
                print_der(pem.contents(), 1, &ctx);
            }
        } else {
            ctx.off_width = (content.len() as f32).log10().floor() as usize + 1;
            print_der(&content, 1, &ctx);
        }
    }

    Ok(())
}

fn print_der(i: &[u8], depth: usize, ctx: &Context) {
    let input = Input::from(i);
    let start = input.start();
    match Any::parse_der(input) {
        Ok((rem, any)) => {
            print_der_any(start, any, depth, ctx);
            if !rem.is_empty() {
                let warning = format!("WARNING: {} extra bytes after object", rem.len());
                indent_println!(depth, "{}", warning.bright_red());
                print_hex_dump(depth, rem.as_bytes2(), ctx);
            }
        }
        Err(e) => {
            eprintln!("Error while parsing at depth {}: {:?}", depth, e);
        }
    }
}

fn print_der_any(start: usize, any: Any, depth: usize, ctx: &Context) {
    if ctx.dump_header {
        let raw_tag = any.header.raw_header().unwrap();
        print!("<");
        for b in raw_tag.as_bytes2() {
            print!("{b:02X} ");
        }
        if ctx.dump_hex_data {
            let max_bytes = min(any.data.len(), 32);
            let bytes = &any.data.as_bytes2()[..max_bytes];
            for b in bytes {
                print!("{b:02X} ");
            }
            if any.data.len() > bytes.len() {
                print!("...");
            }
        }
        println!(">");
    }

    print_offsets(start, any.data.len(), ctx);
    print_header(&any.header, depth, ctx);

    let inner_start = any.data.start();
    match any.header.class() {
        Class::Universal => {
            println!();
        }
        Class::ContextSpecific | Class::Application => {
            let tag_desc = if any.header.class() == Class::Application {
                " APPLICATION"
            } else {
                ""
            };
            // attempt to decode inner object (if EXPLICIT or APPLICATION)
            match Any::parse_der(any.data.clone()) {
                Ok((rem2, inner)) if rem2.is_empty() => {
                    indent_println!(
                        depth + 1,
                        "{} (rem.len={})",
                        format!("EXPLICIT{} [{}]", tag_desc, any.header.tag().0).green(),
                        // any.header.tag.0,
                        rem2.len()
                    );
                    print_der_any(inner_start, inner, depth + 2, ctx);
                }
                _ => {
                    println!();
                    print_hex_dump(depth + 1, any.data.as_bytes2(), ctx);
                }
            }
            return;
        }
        Class::Private => {
            indent_println!(
                depth + 1,
                "PRIVATE: [{}] {}",
                any.header.tag().0,
                "*NOT SUPPORTED*".red()
            );
            return;
        }
    }
    match any.header.tag() {
        Tag::BitString => {
            let span = any.data.span().clone();
            let b = any.bitstring().unwrap();
            // BIT STRING is sometimes used to encapsulate data
            // Attempt to decode it
            let input = Input::new(b.as_raw_slice(), span);
            let mut encapsulates = false;
            if let Ok((rem, inner)) = Any::parse_der(input) {
                if rem.is_empty() {
                    {
                        encapsulates = true;
                        print_offsets_none(ctx);
                        indent_println!(depth + 1, "encapsulates:");
                        print_der_any(inner_start, inner, depth + 1, ctx);
                    }
                }
            }
            // else print it
            if !encapsulates {
                let bit_slice = b.as_bitslice();
                if bit_slice.len() < 64 {
                    let s: String = bit_slice
                        .iter()
                        .rev()
                        .map(|bitref| if *bitref { '1' } else { '0' })
                        .collect();
                    print_offsets_none(ctx);
                    indent_println!(depth + 1, "'{}'B", s);
                } else {
                    // bitstring too long, print as hex
                    print_hex_dump(depth + 1, b.as_ref(), ctx);
                }
            }
        }
        Tag::Boolean => {
            let b = any.bool().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "BOOLEAN: {}", b.to_string().green());
        }
        Tag::EmbeddedPdv => {
            let e = any.embedded_pdv().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "EMBEDDED PDV: {:?}", e);
            print_hex_dump(depth + 1, e.data_value, ctx);
        }
        Tag::Enumerated => {
            let i = any.enumerated().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "ENUMERATED: {}", i.0);
        }
        Tag::GeneralizedTime => {
            let s = any.generalizedtime().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "GeneralizedTime: {}", s);
        }
        Tag::GeneralString => {
            let s = any.generalstring().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "GeneralString: {}", s.as_ref());
        }
        Tag::Ia5String => {
            let s = any.ia5string().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "IA5String: {}", s.as_ref());
        }
        Tag::Integer => {
            let i = any.integer().unwrap();
            match i.as_i128() {
                Ok(i) => {
                    print_offsets_none(ctx);
                    indent_println!(depth + 1, "{}", i);
                }
                Err(_) => {
                    print_hex_dump(depth + 1, i.as_ref(), ctx);
                }
            }
        }
        Tag::Null => (),
        Tag::OctetString => {
            // test if OCTET STRING is encapsulating data
            let data = any.data.clone();
            match Any::parse_der(data) {
                Ok((rem, inner)) if rem.is_empty() => {
                    print_offsets_none(ctx);
                    indent_println!(depth + 1, "encapsulates:");
                    print_der_any(inner_start, inner, depth + 1, ctx);
                }
                _ => {
                    let b = any.octetstring().unwrap();
                    print_hex_dump(depth + 1, b.as_ref(), ctx);
                }
            }
        }
        Tag::Oid => {
            let oid = any.oid().unwrap();
            // let der_oid = DerOid::new(oid.as_bytes().into());
            // let s = format_oid(&der_oid, &ctx.oid_registry).cyan();
            let s = oid.to_string().cyan();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "OID: {}", s);
        }
        Tag::PrintableString => {
            let s = any.printablestring().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "PrintableString: {}", s.as_ref());
        }
        Tag::RelativeOid => {
            let oid = any.oid().unwrap();
            // let der_oid = DerOid::new(oid.as_bytes().into());
            // let s = format_oid(&der_oid, &ctx.oid_registry).cyan();
            let s = oid.to_string().cyan();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "RELATIVE-OID: {}", s);
        }
        Tag::Set => {
            let item_depth = depth + 1;
            for r in AnyIterator::<DerMode>::new(any.data.clone()) {
                //
                match r {
                    Ok((item_input, item)) => {
                        print_der_any(item_input.start(), item, item_depth, ctx);
                    }
                    Err(e) => {
                        eprintln!("Error while parsing at depth {}: {:?}", item_depth, e);
                    }
                }
            }
        }
        Tag::Sequence => {
            let item_depth = depth + 1;
            for r in AnyIterator::<DerMode>::new(any.data.clone()) {
                //
                match r {
                    Ok((item_input, item)) => {
                        print_der_any(item_input.start(), item, item_depth, ctx);
                    }
                    Err(e) => {
                        eprintln!("Error while parsing at depth {}: {:?}", item_depth, e);
                    }
                }
            }
        }
        Tag::UtcTime => {
            let s = any.utctime().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "UtcTime: {}", s);
        }
        Tag::Utf8String => {
            let s = any.utf8string().unwrap();
            print_offsets_none(ctx);
            indent_println!(depth + 1, "UTF-8: {}", s.as_ref());
        }
        _ => unimplemented!("unsupported tag {}", any.header.tag()),
    }
}

fn str_of_length(l: Length) -> String {
    match l {
        Length::Definite(l) => l.to_string(),
        Length::Indefinite => "Indefinite".to_string(),
    }
}
