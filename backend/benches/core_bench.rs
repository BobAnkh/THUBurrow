use backend::utils::{burrow_valid, email};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn valid_burrow_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_burrow_list");
    for burrow in ["", "1", "1,2", "1,2,3", "1,2,3,4,5", "1,2,3,4,5,6,7,8,9,10"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(burrow),
            &burrow,
            |b, &burrow| {
                b.iter(|| burrow_valid::get_burrow_list(burrow));
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("is_valid_burrow");
    for burrow in ["", "1", "1,2", "1,2,3", "1,2,3,4,5", "1,2,3,4,5,6,7,8,9,10"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(burrow),
            &burrow,
            |b, &burrow| {
                b.iter(|| burrow_valid::is_valid_burrow(burrow, &5));
            },
        );
    }
    group.finish();
}

fn check_email_syntax(c: &mut Criterion) {
    let mut group = c.benchmark_group("check_email_syntax");
    for email in [
        "",
        "a",
        "a@",
        "a@b",
        "a@b.",
        "test@mails.tsinghua.edu.cn",
        "@mails.tsinghua.edu.cn",
        "test@163.com",
        "test()@mails.tsinghua.edu.cn",
        "sys-learn2018@tsinghua.edu.cn",
        "shetuan@mail.tsinghua.edu.cn",
    ]
    .iter()
    {
        group.bench_with_input(BenchmarkId::from_parameter(email), &email, |b, &email| {
            b.iter(|| email::check_email_syntax(email));
        });
    }
}

fn id_generator(c: &mut Criterion) {
    backend::setup::id_generator::init(10);
    let mut group = c.benchmark_group("id_generator");
    group.bench_function("id", |b| b.iter(idgenerator::IdInstance::next_id));
    group.finish();
}

fn sign(c: &mut Criterion) {
    let mut group = c.benchmark_group("sign");
    let input = (b"key", b"msg");
    group.bench_with_input(BenchmarkId::new("id", 0), &input, |b, &input| {
        b.iter(|| email::sign(input.0, input.1));
    });
    group.finish();
}

fn assemble_headers(c: &mut Criterion) {
    let mut group = c.benchmark_group("assenmble_headers");
    let timestamp = "1638791815".to_string();
    group.bench_with_input(
        BenchmarkId::from_parameter(timestamp.clone()),
        &(timestamp.clone()),
        |b, timestamp| {
            b.iter(|| email::assemble_headers(timestamp.to_string()));
        },
    );
    group.finish();
}

fn gen_payload(c: &mut Criterion) {
    let mut group = c.benchmark_group("gen_payload");
    let input = email::Body {
        from_email_address: "THUBurrow <noreply@testmail.thuburrow.com>".to_string(),
        destination: vec!["abc@qq.com".to_string()],
        template: email::Template {
            template_id: 21517,
            template_data: format!("{{\\\"code\\\":\"{}\"}}", "abc123"),
        },
        subject: "Verification Email".to_string(),
    };
    group.bench_with_input(BenchmarkId::new("id", 0), &input, |b, input| {
        b.iter(|| email::get_payload(input));
    });
    group.finish();
}

fn signature(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature");
    let param = email::Body {
        from_email_address: "THUBurrow <noreply@testmail.thuburrow.com>".to_string(),
        destination: vec!["abc@qq.com".to_string()],
        template: email::Template {
            template_id: 21517,
            template_data: format!("{{\\\"code\\\":\"{}\"}}", "abc123"),
        },
        subject: "Verification Email".to_string(),
    };
    let timestamp = "1638791815".to_string();
    let date = "2021-12-06".to_string();
    let input = (&param, timestamp, date);
    group.bench_with_input(BenchmarkId::new("id", 0), &input, |b, input| {
        b.iter(|| email::signature(input.0, input.clone().1, input.clone().2));
    });
    group.finish();
}

criterion_group!(
    benches,
    sign,
    assemble_headers,
    gen_payload,
    signature,
    valid_burrow_list,
    check_email_syntax,
    id_generator
);
criterion_main!(benches);
