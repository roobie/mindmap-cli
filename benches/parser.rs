use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mindmap_cli::{Mindmap, parse_node_line};
use std::io::Cursor;

fn bench_parse_node_line(c: &mut Criterion) {
    let line =
        "[123] **AE: Example Node** - This is a description with [456] and [789] references.";
    c.bench_function("parse_node_line", |b| {
        b.iter(|| {
            let _ = parse_node_line(black_box(line), 0);
        })
    });
}

fn bench_mindmap_from_string(c: &mut Criterion) {
    let content = r#"[1] **AE: First** - Description one [2]
[2] **AE: Second** - Description two [3]
[3] **AE: Third** - Description three"#;
    c.bench_function("mindmap_from_string", |b| {
        b.iter(|| {
            Mindmap::load_from_reader(
                black_box(Cursor::new(content)),
                std::path::PathBuf::from("-"),
            )
        })
    });
}

criterion_group!(benches, bench_parse_node_line, bench_mindmap_from_string);
criterion_main!(benches);
