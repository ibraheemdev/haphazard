use haphazard::*;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::{Arc, Barrier};
use std::time::Instant;

macro_rules! folly_bench {
    ($name:ident, $iter:block) => {
        pub fn $name(c: &mut Criterion) {
            let mut group = c.benchmark_group(stringify!($name));
            for nthreads in [1, 2, 4, 8] {
                group.bench_with_input(
                    BenchmarkId::from_parameter(nthreads),
                    &nthreads,
                    |b, &nthreads| {
                        b.iter_custom(|niters| {
                            let barrier = Arc::new(Barrier::new(nthreads + 1));
                            let threads: Vec<_> = (0..nthreads)
                                .map(|_tid| {
                                    let barrier = Arc::clone(&barrier);
                                    std::thread::spawn(move || {
                                        barrier.wait();
                                        barrier.wait();
                                        for _ in 0..(niters / nthreads as u64) {
                                            $iter
                                        }
                                    })
                                })
                                .collect();
                            barrier.wait();
                            let start = Instant::now();
                            barrier.wait();
                            for thread in threads {
                                thread.join().unwrap();
                            }
                            Domain::global().cleanup();
                            start.elapsed()
                        })
                    },
                );
            }
        }
    };
}

folly_bench!(concurrent_new_holder, {
    black_box(HazardPointer::make_global());
});
folly_bench!(concurrent_retire, {
    let foo = Box::into_raw(Box::new(HazPtrObjectWrapper::with_global_domain(0)));
    black_box(unsafe { HazPtrObjectWrapper::retire(foo, &deleters::drop_box) });
});

criterion_group!(benches, concurrent_new_holder, concurrent_retire);
criterion_main!(benches);
