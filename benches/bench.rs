use criterion::*;

// use edom::EDOM;

struct CachedValue<T> {
    value: std::cell::UnsafeCell<Option<T>>
}

impl<T> CachedValue<T> {
    fn new(v:Option<T>)->Self {
        Self {value: std::cell::UnsafeCell::new(v)}
    }
    fn get<'a,F>(&self, f : F)->&'a T where F:FnOnce()->T {
        let v=unsafe {&mut *self.value.get()};
        if v.is_none() {
            *v=Some(f());
        }
        return v.as_ref().unwrap();
    }
}

enum BoxOrNum {
    Num(u64),
    Box(Box<Self>)
}

const NN: usize=10000;


fn bench_box(c: &mut Criterion) {
    const N: usize = NN;
    c.bench_function("bench_box_10000", |b| b.iter(|| {
        let mut r=BoxOrNum::Num(12);
        for i in 0..N {
            r=BoxOrNum::Box(Box::new(r));
        }
    }));
}

fn bench_cached_value_push(c: &mut Criterion) {
    const N: usize = NN;
    c.bench_function("bench_cached_value_push_10000", |b| b.iter(|| {
        let mut vcv:Vec<CachedValue<u64>>=Vec::new();
        vcv.push(CachedValue::new(Some(13)));
        for i in 1..N {
            vcv.push(CachedValue::new(None));
        }
    }));
}


fn bench_cached_value_push_get(c: &mut Criterion) {
    const N: usize = NN;
    c.bench_function("bench_cached_value_push_10000", |b| b.iter(|| {
        let mut vcv:Vec<CachedValue<u64>>=Vec::new();
        vcv.push(CachedValue::new(Some(13)));
        for i in 1..N {
            vcv.push(CachedValue::new(None));
        }
        for i in 0..N {
            vcv[i].get(|| *vcv[i-1].get(|| 2));
        }
       
    }));
}



fn bench_cached_value_easy_get(c: &mut Criterion) {
    const N: usize = NN;
    c.bench_function("bench_cached_value_easy_get", |b| b.iter(|| {
        let mut vcv:Vec<CachedValue<u64>>=Vec::new();
        for _ in 0..N {
            vcv.push(CachedValue::new(Some(13)));
        }
        for i in 0..N {
            vcv[i].get(|| *vcv[i-1].get(|| 2));
        }
       
    }));
}

fn bench_fold(c: &mut Criterion) {
    const N: usize = NN;
    c.bench_function("bench_fold", |b| b.iter(|| {
        (0..NN).fold(0, |x, y| x + y)
    }));
}

fn bench_vec(c: &mut Criterion) {
    const N: usize = NN;
    c.bench_function("bench_vec", |b| b.iter(|| {
        vec![0u8; N]
    }));
}

use edom::*;
use rand::prelude::*;

/*
fn bench_js_framework_benchmark_create10000(c: &mut Criterion) {
    const N: usize = NN;
    
    c.bench_function("create_10000", |b| b.iter(|| {
        let mut v:Vec<(u32,String)>=Vec::new();
        let mut n=1;
        let mut selected : Option<u32>=None;
        let mut thread_rng = thread_rng();
        for _ in 0..10000 {
                v.push(next_line(&mut n, &mut thread_rng));
        }
        edom::EDOM::render("body", edom::NoopElementNode {tag:"body"}, move |mut root| {
            js_framework_benchmark(&mut root, &mut v, &mut n, &mut thread_rng, &mut selected);
        });
    }));
}
 */


criterion_group!{
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.9).sample_size(10);
    targets = bench_box, bench_fold, bench_vec, bench_cached_value_push, bench_cached_value_push_get, bench_cached_value_easy_get
}
criterion_main!(benches);