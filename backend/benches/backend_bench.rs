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
    backend::utils::id_gen::init(10);
    let mut group = c.benchmark_group("id_generator");
    group.bench_function("id", |b| b.iter(|| idgenerator::IdHelper::next_id()));
    group.finish();
}

criterion_group!(benches, valid_burrow_list, check_email_syntax, id_generator);
criterion_main!(benches);
