use cs_parser::ClassFile;
use std::io;
use std::io::Write;

pub fn display_class<W: Write>(mut w: W, class: &ClassFile) -> Result<(), io::Error> {
    let cp = &class.constant_pool;

    writeln!(
        w,
        ".class ({:#X?}) file version {}.{}",
        class.magic, class.major_version, class.minor_version
    )?;

    writeln!(w)?;

    writeln!(
        w,
        "class {} extends {}{} {{",
        &class.this_class.get(cp).name_index.get(cp),
        match class.super_class.maybe_get(cp) {
            None => "<none>",
            Some(class) => class.name_index.get(cp),
        },
        if class.interfaces.is_empty() {
            "".to_string()
        } else {
            format!(
                " implements {}",
                class
                    .interfaces
                    .iter()
                    .map(|i| i.get(cp))
                    .map(|i| i.name_index.get(cp))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        },
    )?;

    writeln!(w, " Attributes:")?;
    for attr in &class.attributes {
        writeln!(w, "  {}", &attr.attribute_name_index.get(cp))?;
    }
    writeln!(w)?;

    writeln!(w, " Fields:")?;
    for field in &class.fields {
        writeln!(
            w,
            "  {} {}",
            &field.descriptor_index.get(cp),
            &field.name_index.get(cp)
        )?;
    }
    writeln!(w)?;

    writeln!(w, " Methods:")?;
    for method in &class.methods {
        writeln!(
            w,
            "  {} {}",
            &method.descriptor_index.get(cp),
            &method.name_index.get(cp),
        )?;
    }

    writeln!(w, "}}")?;
    Ok(())
}
