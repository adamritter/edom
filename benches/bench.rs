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
        for _ in 0..N {
            r=BoxOrNum::Box(Box::new(r));
        }
    }));
}

fn bench_cached_value_push(c: &mut Criterion) {
    const N: usize = NN;
    c.bench_function("bench_cached_value_push_10000", |b| b.iter(|| {
        let mut vcv:Vec<CachedValue<u64>>=Vec::new();
        vcv.push(CachedValue::new(Some(13)));
        for _ in 1..N {
            vcv.push(CachedValue::new(None));
        }
    }));
}


fn bench_cached_value_push_get(c: &mut Criterion) {
    const N: usize = NN;
    c.bench_function("bench_cached_value_push_10000", |b| b.iter(|| {
        let mut vcv:Vec<CachedValue<u64>>=Vec::new();
        vcv.push(CachedValue::new(Some(13)));
        for _ in 1..N {
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

fn vdom_create_10000_with_30_elems(c: &mut Criterion) {
    c.bench_function("vdom_create_10000_with_30_elems", |b| b.iter(|| {
        let mut v:Vec<(u32,String)>=Vec::new();
        for i in 0..10000 {
                v.push((i, i.to_string()));
        }
        edom::EDOM::render(edom::noop::ElementNode {tag:"body", generic_node: noop::Node {  }}, 
                move |mut root| {
            root.for_each(v.iter_mut(), |(i, _)| *i, "span", 
                    |elem, span| {
                span.text(elem.1.as_str());
                for _ in 0..30 {
                    span.h1();
                }
            })
        });
    }));
}

fn vdom_create_10000_with_30_elems_swap20x(c: &mut Criterion) {
    c.bench_function("vdom_create_10000_with_30_elems_swap20x", |b| b.iter(|| {
        let mut v:Vec<(u32,String)>=Vec::new();
        for i in 0..10000 {
                v.push((i, i.to_string()));
        }
        let e=edom::EDOM::render(
                edom::noop::ElementNode {tag:"body", generic_node: noop::Node {  }}, 
                move |mut root| {
            root.for_each(v.iter_mut(), |(i, _)| *i, "span", 
                    |elem, span| {
                span.text(elem.1.as_str());
                for _ in 0..30 {
                    span.h1();
                }
            });
            v.swap(1, 9999);
        });
        let fire_event=(*(e.borrow())).fire_event.clone();
        for _ in 1..20 {
            (*((*(fire_event)).borrow_mut()))(0, "noevent".to_string(), noop::Event {  });
        }
    }));
}

criterion_group!{
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.9).sample_size(10);
    targets = bench_box, bench_fold, bench_vec, bench_cached_value_push, bench_cached_value_push_get,
            bench_cached_value_easy_get, vdom_create_10000_with_30_elems, vdom_create_10000_with_30_elems_swap20x
}
criterion_main!(benches);