use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use bindgen::Builder;

use ::{EventClass, EventInstance, Field, Provider};
use super::ctf_field_c_type;

pub(in super) fn generate_interface_impl(path: &PathBuf,
                                         providers: &[Provider],
                                         interface_header: &PathBuf,
                                         tracepoint_header: &PathBuf) -> io::Result<()> {
    let mut outf = File::create(path)
        .expect(&format!("Failed to create tracepoint interface implementation {:?}\n", path));

    write!(outf, "#include \"{}\"\n", interface_header.to_string_lossy())?;
    write!(outf, "#include \"{}\"\n", tracepoint_header.to_string_lossy())?;

    for provider in providers {
        generate_provider_impl(provider, &mut outf)?;
    }

    Ok(())
}

pub(in super) fn generate_interface_header(path: &PathBuf, providers: &[Provider]) -> io::Result<()> {
    let mut outf = File::create(path)
        .expect(&format!("Failed to create tracepoint interface header {:?}\n", path));

    write!(outf, "#if !defined(_RUST_TRACEPOINT_INTERFACE)\n")?;
    write!(outf, "#define _RUST_TRACEPOINT_INTERFACE\n")?;
    write!(outf, "#include <stdint.h>\n")?;
    write!(outf, "#include <stddef.h>\n")?;

    for provider in providers {
        generate_provider_header(provider, &mut outf)?;
    }

    write!(outf, "#endif")?;
    Ok(())
}

pub(in super) fn whitelist_interface(providers: &[Provider], mut b: Builder) -> Builder {
    for provider in providers {
        for event_class in &provider.classes {
            for instance in &event_class.instances {
                let fname = generate_func_name(provider, event_class, instance);
                eprintln!("whitelisting: {}", fname);
                b = b.allowlist_function(fname);
            }
        }
    }
    b
}

fn generate_provider_impl<F: Write>(provider: &Provider, outf: &mut F) -> io::Result<()> {
    for event_class in &provider.classes {
        for instance in &event_class.instances {
            write!(outf, "void {}(", generate_func_name(provider, event_class, instance))?;
            generate_c_args(&event_class.fields, outf, true)?;
            write!(outf, ") {{\n")?;
            write!(outf, "    tracepoint({}, {}, ", provider.name, instance.name)?;
            generate_c_args(&event_class.fields, outf, false)?;
            write!(outf, ");\n")?;
            write!(outf, "}}\n\n")?;
        }
    }

    Ok(())
}

fn generate_provider_header<F: Write>(provider: &Provider, outf: &mut F) -> io::Result<()> {
    for event_class in &provider.classes {
        for instance in &event_class.instances {
            write!(outf, "extern void {}(", generate_func_name(provider, event_class, instance))?;
            generate_c_args(&event_class.fields, outf, true)?;
            write!(outf, ");\n")?;
        }
    }

    Ok(())
}

pub fn generate_func_name(provider: &Provider, event_class: &EventClass, instance: &EventInstance) -> String {
    format!(
        "{}_{}_{}_tp",
        provider.name,
        event_class.class_name,
        instance.name
    )
}

fn generate_c_args<F: Write>(fields: &[Field], outf: &mut F, include_type: bool) -> io::Result<()> {
    let mut first = true;
    for field in fields {
        if first {
            first = false
        } else {
            write!(outf, ", ")?;
        }
        if include_type {
            write!(
                outf, "{} {}_arg",
                ctf_field_c_type(field.ctf_type),
                field.name
            )?;
            if field.ctf_type.is_sequence() {
                write!(outf, ", size_t {}_len", field.name)?;
            }
        } else {
            write!(outf, "{}_arg", field.name)?;
            if field.ctf_type.is_sequence() {
                write!(outf, ", {}_len", field.name)?;
            }
        }
    }
    Ok(())
}
