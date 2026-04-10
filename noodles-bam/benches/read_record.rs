use std::hint::black_box;
use std::num::NonZero;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use noodles_bam as bam;
use noodles_core::Position;
use noodles_sam::{
    self as sam,
    alignment::{
        RecordBuf,
        io::Write as _,
        record::{
            Flags, MappingQuality,
            cigar::{Op, op::Kind},
            data::field::Tag,
        },
        record_buf::{Cigar, Sequence, data::field::Value},
    },
    header::record::value::{Map, map::ReferenceSequence},
};

const NUM_RECORDS: usize = 10_000;

fn build_cigar(i: usize, read_len: usize) -> Cigar {
    let bucket = i % 100;

    if bucket < 85 {
        // 85%: perfect match
        Cigar::from(vec![Op::new(Kind::Match, read_len)])
    } else if bucket < 90 {
        // 5%: 5' soft clip
        Cigar::from(vec![
            Op::new(Kind::SoftClip, 5),
            Op::new(Kind::Match, read_len - 5),
        ])
    } else if bucket < 95 {
        // 5%: 3' soft clip
        Cigar::from(vec![
            Op::new(Kind::Match, read_len - 5),
            Op::new(Kind::SoftClip, 5),
        ])
    } else if bucket < 98 {
        // 3%: single insertion
        let half = read_len / 2;
        Cigar::from(vec![
            Op::new(Kind::Match, half),
            Op::new(Kind::Insertion, 1),
            Op::new(Kind::Match, read_len - half - 1),
        ])
    } else {
        // 2%: single deletion
        let half = read_len / 2;
        Cigar::from(vec![
            Op::new(Kind::Match, half),
            Op::new(Kind::Deletion, 1),
            Op::new(Kind::Match, read_len - half),
        ])
    }
}

fn build_md(i: usize, read_len: usize) -> Value {
    let bucket = i % 100;

    if bucket < 85 {
        Value::from(format!("{read_len}"))
    } else if bucket < 95 {
        // soft clips don't appear in MD
        let matched = read_len - 5;
        Value::from(format!("{matched}"))
    } else if bucket < 98 {
        // insertion: MD reflects only ref-consuming ops
        let half = read_len / 2;
        Value::from(format!("{}{}", half, read_len - half - 1))
    } else {
        // deletion
        let half = read_len / 2;
        Value::from(format!("{half}^A{half}"))
    }
}

fn build_mate_cigar(i: usize, read_len: usize) -> Value {
    let bucket = i % 100;

    if bucket < 90 {
        Value::from(format!("{read_len}M"))
    } else {
        Value::from(format!("5S{}M", read_len - 5))
    }
}

/// Generates a BGZF-compressed BAM byte stream with `n` records of `read_len` bases each,
/// using realistic field values representative of BWA-MEM paired-end WGS output.
fn build_bam_data(n: usize, read_len: usize) -> (Vec<u8>, sam::Header) {
    let header = sam::Header::builder()
        .add_reference_sequence(
            "chr1",
            Map::<ReferenceSequence>::new(NonZero::new(249_250_621).unwrap()),
        )
        .build();

    let mut writer = bam::io::Writer::new(Vec::new());
    writer.write_header(&header).unwrap();

    let seq = Sequence::from(
        (0..read_len)
            .map(|i| b"ACGTACGT"[i % 8])
            .collect::<Vec<u8>>(),
    );
    let qual: Vec<u8> = (0..read_len).map(|i| (20 + (i % 21)) as u8).collect();

    let flags = Flags::SEGMENTED
        | Flags::PROPERLY_SEGMENTED
        | Flags::MATE_REVERSE_COMPLEMENTED
        | Flags::FIRST_SEGMENT;

    let mq_tag = Tag::new(b'M', b'Q');

    for i in 0..n {
        let pos = 1 + (i % 100_000);
        let nm: u8 = if i % 100 >= 98 { 1 } else { 0 };

        let record = RecordBuf::builder()
            .set_name(format!("HISEQ:1:2103:{i}:81321"))
            .set_flags(flags)
            .set_reference_sequence_id(0)
            .set_alignment_start(Position::try_from(pos).unwrap())
            .set_mapping_quality(MappingQuality::try_from(60).unwrap())
            .set_cigar(build_cigar(i, read_len))
            .set_mate_reference_sequence_id(0)
            .set_mate_alignment_start(Position::try_from(pos + 300).unwrap())
            .set_template_length(300)
            .set_sequence(seq.clone())
            .set_quality_scores(qual.iter().copied().collect())
            .set_data(
                [
                    (Tag::ALIGNMENT_SCORE, Value::UInt8(read_len as u8)),
                    (mq_tag, Value::UInt8(60)),
                    (Tag::EDIT_DISTANCE, Value::UInt8(nm)),
                    (Tag::ALIGNMENT_HIT_COUNT, Value::UInt8(1)),
                    (Tag::READ_GROUP, Value::from("SampleA_LibB_Lane3")),
                    (Tag::MISMATCHED_POSITIONS, build_md(i, read_len)),
                    (Tag::MATE_CIGAR, build_mate_cigar(i, read_len)),
                ]
                .into_iter()
                .collect(),
            )
            .build();

        writer.write_alignment_record(&header, &record).unwrap();
    }

    writer.try_finish().unwrap();
    let data = writer.get_ref().get_ref().clone();
    (data, header)
}

fn bench_read_record(c: &mut Criterion) {
    for read_len in [150, 300] {
        let mut group = c.benchmark_group(format!("{read_len}bp"));
        let (bam_data, header) = build_bam_data(NUM_RECORDS, read_len);
        group.throughput(Throughput::Elements(NUM_RECORDS as u64));

        group.bench_function("record_buf", |b| {
            b.iter(|| {
                let mut reader = bam::io::Reader::new(bam_data.as_slice());
                let _ = reader.read_header().unwrap();
                let mut record = RecordBuf::default();
                let mut n = 0u64;
                while reader.read_record_buf(&header, &mut record).unwrap() > 0 {
                    black_box(&record);
                    n += 1;
                }
                n
            })
        });

        group.bench_function("record_lazy", |b| {
            b.iter(|| {
                let mut reader = bam::io::Reader::new(bam_data.as_slice());
                let _ = reader.read_header().unwrap();
                let mut record = bam::Record::default();
                let mut n = 0u64;
                while reader.read_record(&mut record).unwrap() > 0 {
                    black_box(&record);
                    n += 1;
                }
                n
            })
        });

        group.finish();
    }
}

criterion_group!(benches, bench_read_record);
criterion_main!(benches);
