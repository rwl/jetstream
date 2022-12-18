all: bench

BENCHMARK = encoder_benchmark
BASELINE = master

.PHONY: bench
bench:
	cargo criterion

.PHONY: save
save:
	cargo bench --bench $(BENCHMARK) -- --save-baseline $(BASELINE)

.PHONY: baseline
baseline:
	cargo bench --bench $(BENCHMARK) -- --baseline $(BASELINE)
